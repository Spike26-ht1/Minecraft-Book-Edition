package dev.bookedition.game

import android.app.Activity
import android.app.AlertDialog
import android.graphics.BitmapFactory
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.content.Intent
import android.opengl.EGL14
import android.opengl.GLSurfaceView
import android.os.Bundle
import android.util.Base64
import android.view.Gravity
import android.view.View
import android.view.WindowManager
import android.widget.EditText
import android.widget.FrameLayout
import android.widget.ImageView
import android.widget.LinearLayout
import android.widget.TextView
import java.nio.charset.StandardCharsets
import java.security.SecureRandom

/** Game entry point embedded in the Book Edition launcher APK. */
class MainActivity : Activity() {
    private var gameSurface: GLSurfaceView? = null
    private var showingGame = false
    private var gameRenderer: BookRenderer? = null
    private var gameBridge: NativeBridge? = null
    private val resourcePackRequestCode = 501
    private var selectedWorldSeed: Long = 20261116L

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        requestedOrientation = android.content.pm.ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        selectedWorldSeed = getSharedPreferences("book_worlds", MODE_PRIVATE)
            .getLong("selected_seed", 20261116L)
        hideSystemUi()
        showMainMenu()
    }

    override fun onResume() {
        super.onResume()
        gameSurface?.onResume()
        hideSystemUi()
    }

    override fun onPause() {
        gameBridge?.saveWorld()
        gameSurface?.onPause()
        super.onPause()
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (hasFocus) hideSystemUi()
    }

    @Deprecated("Returns from the world scene to the game menu.")
    override fun onBackPressed() {
        if (showingGame) {
            gameBridge?.saveWorld()
            gameSurface?.onPause()
            gameSurface = null
            gameRenderer = null
            gameBridge = null
            showingGame = false
            showMainMenu()
        } else {
            super.onBackPressed()
        }
    }

    private fun hideSystemUi() {
        @Suppress("DEPRECATION")
        window.decorView.systemUiVisibility = (
            View.SYSTEM_UI_FLAG_FULLSCREEN
                or View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                or View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                or View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                or View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                or View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
            )
    }

    private fun showMainMenu() {
        showingGame = false
        val root = FrameLayout(this).apply {
            background = GradientDrawable(
                GradientDrawable.Orientation.TL_BR,
                intArrayOf(Color.rgb(8, 17, 28), Color.rgb(19, 39, 54), Color.rgb(9, 16, 26))
            )
        }
        val content = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER
            setPadding(dp(28), dp(14), dp(28), dp(14))
        }
        val mark = ImageView(this).apply {
            contentDescription = "Símbolo do Minecraft Book Edition"
            scaleType = ImageView.ScaleType.FIT_CENTER
            setImageBitmap(loadProjectMark())
        }
        content.addView(mark, LinearLayout.LayoutParams(dp(76), dp(76)).apply {
            bottomMargin = dp(8)
        })
        content.addView(label("MINECRAFT BOOK EDITION", 22f, Color.WHITE, true))
        content.addView(label("UM MUNDO ABERTO PARA EXPLORAR", 10f, Color.rgb(155, 190, 211), false).apply {
            letterSpacing = 0.18f
            setPadding(0, dp(5), 0, dp(18))
        })
        content.addView(menuButton("JOGAR", true) { showWorldCreation() }, buttonLayoutParams())
        content.addView(menuButton("CONFIGURAÇÕES", false) { showSettings() }, buttonLayoutParams(10))
        content.addView(menuButton("SAIR", false) { finish() }, buttonLayoutParams(10))
        val engineStatus = if (NativeBridge.available) "BOOK64 GLES 3.0" else "MOTOR NATIVO NÃO CARREGADO"
        content.addView(label("PRÉ-ALPHA  •  $engineStatus", 9f, Color.rgb(140, 163, 181), false).apply {
            setPadding(0, dp(18), 0, 0)
        })
        root.addView(content, FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.MATCH_PARENT,
            FrameLayout.LayoutParams.MATCH_PARENT
        ))
        setContentView(root)
        hideSystemUi()
    }

    private fun showWorldCreation() {
        val seedInput = EditText(this).apply {
            inputType = android.text.InputType.TYPE_CLASS_NUMBER or
                android.text.InputType.TYPE_NUMBER_FLAG_SIGNED
            setSingleLine(true)
            setText(selectedWorldSeed.toString())
            hint = "Seed numérica"
        }
        val content = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dp(22), dp(8), dp(22), dp(4))
            addView(label("A mesma seed reproduz o mesmo terreno nesta versão do gerador.", 12f,
                Color.rgb(170, 185, 198), false))
            addView(seedInput, LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT, dp(54)
            ).apply { topMargin = dp(12) })
        }

        AlertDialog.Builder(this)
            .setTitle("Criar mundo")
            .setView(content)
            .setPositiveButton("CRIAR") { _, _ ->
                val seed = seedInput.text.toString().trim().toLongOrNull()
                if (seed == null) {
                    android.widget.Toast.makeText(
                        this, "Digite uma seed numérica válida.", android.widget.Toast.LENGTH_LONG
                    ).show()
                    showWorldCreation()
                    return@setPositiveButton
                }
                selectedWorldSeed = seed
                getSharedPreferences("book_worlds", MODE_PRIVATE)
                    .edit().putLong("selected_seed", seed).apply()
                startGameScene()
            }
            .setNeutralButton("SEED ALEATÓRIA") { _, _ ->
                selectedWorldSeed = SecureRandom().nextLong()
                getSharedPreferences("book_worlds", MODE_PRIVATE)
                    .edit().putLong("selected_seed", selectedWorldSeed).apply()
                startGameScene()
            }
            .setNegativeButton("VOLTAR", null)
            .show()
    }

    private fun startGameScene() {
        if (!NativeBridge.available) {
            AlertDialog.Builder(this)
                .setTitle("Motor gráfico indisponível")
                .setMessage("A biblioteca nativa Book64 não foi carregada. Esta instalação não contém o runtime ARM64 necessário.")
                .setPositiveButton("OK", null)
                .show()
            return
        }
        showingGame = true
        val root = FrameLayout(this).apply {
            setBackgroundColor(Color.rgb(9, 14, 23))
        }
        val bridge = NativeBridge()
        gameBridge = bridge
        val saveDirectory = getExternalMediaDirs().firstOrNull()
            ?.let { java.io.File(it, "book-edition/worlds") }
            ?.takeIf { it.exists() || it.mkdirs() }
            ?: java.io.File(filesDir, "book_worlds").apply { mkdirs() }
        val worldSavePath = java.io.File(saveDirectory, "world_${selectedWorldSeed}.bookworld").absolutePath
        var hud: GameHud? = null
        val initialUncapped = getSharedPreferences("book_settings", MODE_PRIVATE)
            .getBoolean("uncapped_fps", false)
        val renderer = BookRenderer(bridge, initialUncapped, selectedWorldSeed, worldSavePath) { currentFps, frameMs, mode ->
            hud?.updateFrame(currentFps, frameMs, mode)
        }
        gameRenderer = renderer
        val surface = GLSurfaceView(this).apply {
            setEGLContextClientVersion(3)
            preserveEGLContextOnPause = false
            setRenderer(renderer)
            renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY
        }
        gameSurface = surface
        root.addView(surface, FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.MATCH_PARENT,
            FrameLayout.LayoutParams.MATCH_PARENT
        ))
        hud = GameHud(this, root, bridge) { hud?.let { showGameSettings(it) } }
        hud?.attach()
        root.addView(label("MUNDO LOCAL  •  PRÉ-ALPHA", 9f, Color.WHITE, false).apply {
            setPadding(dp(12), dp(8), dp(12), dp(8))
            background = rounded(Color.argb(160, 12, 19, 29), dp(10))
        }, FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.WRAP_CONTENT,
            FrameLayout.LayoutParams.WRAP_CONTENT,
            Gravity.BOTTOM or Gravity.CENTER_HORIZONTAL
        ).apply { setMargins(dp(18), 0, dp(18), dp(16)) })
        setContentView(root)
        hideSystemUi()
    }

    private fun showSettings() {
        AlertDialog.Builder(this)
            .setTitle("Configurações de teste")
            .setMessage(
                "Renderizador: OpenGL ES 3.0\n" +
                    "Plataforma: Android ARM64\n\n" +
                    "Entre na cena de teste para abrir o menu mobile. Lá você pode alternar o F3, " +
                    "mostrar ou ocultar os controles e ajustar a sensibilidade da câmera."
            )
            .setPositiveButton("OK", null)
            .show()
    }

    private fun showGameSettings(hud: GameHud) {
        val currentFpsMode = gameRenderer?.fpsModeLabel ?: "VSync"
        val options = arrayOf(
            "Alternar painel F3",
            "Mostrar / ocultar controles mobile",
            "Ciclo de sensibilidade da câmera",
            "Limite de FPS: $currentFpsMode",
            "Voltar ao menu principal",
            "Importar Resource Pack (.mcpack / .zip)"
        )
        AlertDialog.Builder(this)
            .setTitle("Configurações de teste")
            .setItems(options) { _, selected ->
                when (selected) {
                    0 -> hud.toggleDebug()
                    1 -> {
                        val visible = hud.toggleControls()
                        android.widget.Toast.makeText(
                            this,
                            if (visible) "Controles ativados" else "Controles ocultos",
                            android.widget.Toast.LENGTH_SHORT
                        ).show()
                    }
                    2 -> {
                        val level = hud.cycleSensitivity()
                        android.widget.Toast.makeText(
                            this,
                            "Sensibilidade: $level",
                            android.widget.Toast.LENGTH_SHORT
                        ).show()
                    }
                    3 -> {
                        val next = !(gameRenderer?.isUncapped ?: false)
                        getSharedPreferences("book_settings", MODE_PRIVATE)
                            .edit().putBoolean("uncapped_fps", next).apply()
                        gameRenderer?.setUncapped(next)
                        gameSurface?.queueEvent { gameRenderer?.applyFramePacing() }
                        android.widget.Toast.makeText(
                            this,
                            if (next) "FPS sem limite solicitado" else "VSync ativado",
                            android.widget.Toast.LENGTH_SHORT
                        ).show()
                    }
                    4 -> returnToMainMenu()
                    5 -> selectResourcePack()
                }
            }
            .show()
    }


    private fun selectResourcePack() {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            putExtra(Intent.EXTRA_MIME_TYPES, arrayOf(
                "application/zip",
                "application/octet-stream",
                "application/x-zip-compressed"
            ))
        }
        startActivityForResult(intent, resourcePackRequestCode)
    }

    @Deprecated("Uses Android's document picker for compatibility.")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode != resourcePackRequestCode || resultCode != RESULT_OK) return
        val uri = data?.data ?: return
        android.widget.Toast.makeText(this, "Lendo Resource Pack…", android.widget.Toast.LENGTH_SHORT).show()
        Thread {
            val result = runCatching { ResourcePackImporter.import(this, uri) }
            runOnUiThread {
                result.onSuccess { report ->
                    android.widget.Toast.makeText(
                        this,
                        "Pack: ${report.name} • ${report.loaded} texturas • ${report.missing} ausentes",
                        android.widget.Toast.LENGTH_LONG
                    ).show()
                }.onFailure { error ->
                    AlertDialog.Builder(this)
                        .setTitle("Falha ao importar Resource Pack")
                        .setMessage(error.message ?: "Arquivo inválido.")
                        .setPositiveButton("OK", null)
                        .show()
                }
            }
        }.start()
    }

    private fun returnToMainMenu() {
        gameBridge?.saveWorld()
        gameSurface?.onPause()
        gameSurface = null
        gameRenderer = null
        gameBridge = null
        showingGame = false
        showMainMenu()
    }

    private fun createBrandPanel(): View {
        val panel = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(dp(12), dp(8), dp(16), dp(8))
            background = rounded(Color.argb(208, 12, 19, 29), dp(16))
        }
        val mark = ImageView(this).apply {
            contentDescription = "Símbolo do Book Edition"
            scaleType = ImageView.ScaleType.FIT_CENTER
            setImageBitmap(loadProjectMark())
        }
        panel.addView(mark, LinearLayout.LayoutParams(dp(44), dp(44)))
        panel.addView(label("BOOK EDITION\nBOOK64 • GLES 3.0", 11f, Color.WHITE, true).apply {
            setPadding(dp(10), 0, 0, 0)
        })
        return panel
    }

    private fun menuButton(text: String, primary: Boolean, action: () -> Unit): TextView =
        TextView(this).apply {
            this.text = text
            gravity = Gravity.CENTER
            textSize = 13f
            letterSpacing = 0.08f
            typeface = Typeface.create("sans-serif-medium", Typeface.BOLD)
            setTextColor(if (primary) Color.rgb(8, 24, 31) else Color.WHITE)
            background = rounded(
                if (primary) Color.rgb(119, 207, 154) else Color.argb(190, 35, 55, 70),
                dp(13)
            )
            setOnClickListener { action() }
            isClickable = true
            isFocusable = true
        }

    private fun buttonLayoutParams(top: Int = 0) =
        LinearLayout.LayoutParams(dp(250), dp(46)).apply { topMargin = dp(top) }

    private fun label(text: String, size: Float, color: Int, bold: Boolean) =
        TextView(this).apply {
            this.text = text
            textSize = size
            setTextColor(color)
            gravity = Gravity.CENTER
            typeface = Typeface.create("sans-serif", if (bold) Typeface.BOLD else Typeface.NORMAL)
        }

    private fun rounded(color: Int, radius: Int) =
        GradientDrawable().apply {
            setColor(color)
            cornerRadius = radius.toFloat()
        }

    private fun loadProjectMark() = runCatching {
        val encoded = assets.open("book-edition-mark.png.base64")
            .bufferedReader(StandardCharsets.UTF_8)
            .use { it.readText() }
        val bytes = Base64.decode(encoded, Base64.DEFAULT)
        BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
    }.getOrNull()

    private fun dp(value: Int): Int = (value * resources.displayMetrics.density).toInt()
}

private class BookRenderer(
    private val bridge: NativeBridge,
    initialUncapped: Boolean,
    private val worldSeed: Long,
    private val worldSavePath: String,
    private val onFrameStats: (Int, Double, String) -> Unit
) : GLSurfaceView.Renderer {
    @Volatile
    var isUncapped: Boolean = initialUncapped
        private set
    @Volatile
    private var appliedFpsMode = if (initialUncapped) "UNCAP solicitado" else "VSync"
    val fpsModeLabel: String get() = appliedFpsMode

    fun setUncapped(enabled: Boolean) {
        isUncapped = enabled
        appliedFpsMode = if (enabled) "UNCAP solicitado" else "VSync solicitado"
    }

    fun applyFramePacing() {
        val display = EGL14.eglGetCurrentDisplay()
        if (display == EGL14.EGL_NO_DISPLAY) {
            appliedFpsMode = "EGL indisponível"
            return
        }
        val applied = EGL14.eglSwapInterval(display, if (isUncapped) 0 else 1)
        appliedFpsMode = when {
            applied && isUncapped -> "UNCAP (swap 0)"
            applied -> "VSync (swap 1)"
            isUncapped -> "UNCAP não suportado"
            else -> "VSync padrão"
        }
    }

    private var lastFrameNanos = 0L
    private var statsStartNanos = System.nanoTime()
    private var framesSinceStats = 0
    private var lastFrameMs = 0.0

    override fun onSurfaceCreated(
        gl: javax.microedition.khronos.opengles.GL10?,
        config: javax.microedition.khronos.egl.EGLConfig?
    ) {
        bridge.surfaceCreated(worldSeed, worldSavePath)
        applyFramePacing()
    }

    override fun onSurfaceChanged(
        gl: javax.microedition.khronos.opengles.GL10?,
        width: Int,
        height: Int
    ) {
        bridge.surfaceChanged(width, height)
    }

    override fun onDrawFrame(gl: javax.microedition.khronos.opengles.GL10?) {
        bridge.drawFrame()
        val now = System.nanoTime()
        if (lastFrameNanos != 0L) {
            lastFrameMs = (now - lastFrameNanos) / 1_000_000.0
        }
        lastFrameNanos = now
        framesSinceStats++
        val elapsed = now - statsStartNanos
        if (elapsed >= 500_000_000L) {
            val currentFps = (framesSinceStats * 1_000_000_000.0 / elapsed).toInt()
            onFrameStats(currentFps, lastFrameMs, fpsModeLabel)
            framesSinceStats = 0
            statsStartNanos = now
        }
    }
}

internal class NativeBridge {
    fun setMovement(forward: Float, strafe: Float) {
        if (available) nativeSetMovement(forward, strafe)
    }

    fun breakBlock() {
        if (available) nativeBreakBlock()
    }

    fun placeBlock() {
        if (available) nativePlaceBlock()
    }

    fun saveWorld() {
        if (available) nativeSaveWorld()
    }

    fun jump() {
        if (available) nativeJump()
    }

    fun look(yawDelta: Float, pitchDelta: Float) {
        if (available) nativeLook(yawDelta, pitchDelta)
    }

    fun debugInfo(): String = if (available) nativeGetDebugInfo() else "Book64 unavailable"

    fun surfaceCreated(seed: Long, savePath: String) {
        if (available) nativeSurfaceCreated(seed, savePath)
    }

    fun surfaceChanged(width: Int, height: Int) {
        if (available) nativeSurfaceChanged(width, height)
    }

    fun drawFrame() {
        if (available) nativeDrawFrame()
    }

    private external fun nativeSurfaceCreated(seed: Long, savePath: String)
    private external fun nativeSurfaceChanged(width: Int, height: Int)
    private external fun nativeDrawFrame()
    private external fun nativeSetMovement(forward: Float, strafe: Float)
    private external fun nativeLook(yawDelta: Float, pitchDelta: Float)
    private external fun nativeJump()
    private external fun nativeBreakBlock()
    private external fun nativePlaceBlock()
    private external fun nativeSaveWorld()
    private external fun nativeGetDebugInfo(): String

    companion object {
        val available: Boolean = runCatching {
            System.loadLibrary("book_android")
            true
        }.getOrDefault(false)
    }
}

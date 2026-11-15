package dev.bookedition.game

import android.app.Activity
import android.app.AlertDialog
import android.graphics.BitmapFactory
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.opengl.GLSurfaceView
import android.os.Bundle
import android.util.Base64
import android.view.Gravity
import android.view.View
import android.view.WindowManager
import android.widget.FrameLayout
import android.widget.ImageView
import android.widget.LinearLayout
import android.widget.TextView
import java.nio.charset.StandardCharsets

/** Game entry point embedded in the Book Edition launcher APK. */
class MainActivity : Activity() {
    private var gameSurface: GLSurfaceView? = null
    private var showingGame = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        requestedOrientation = android.content.pm.ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        hideSystemUi()
        showMainMenu()
    }

    override fun onResume() {
        super.onResume()
        gameSurface?.onResume()
        hideSystemUi()
    }

    override fun onPause() {
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
            gameSurface?.onPause()
            gameSurface = null
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
        content.addView(menuButton("JOGAR", true) { startGameScene() }, buttonLayoutParams())
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

    private fun startGameScene() {
        showingGame = true
        val root = FrameLayout(this).apply {
            setBackgroundColor(Color.rgb(9, 14, 23))
        }
        val surface = GLSurfaceView(this).apply {
            setEGLContextClientVersion(3)
            preserveEGLContextOnPause = false
            setRenderer(BookRenderer())
            renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY
        }
        gameSurface = surface
        root.addView(surface, FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.MATCH_PARENT,
            FrameLayout.LayoutParams.MATCH_PARENT
        ))
        root.addView(createBrandPanel(), FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.WRAP_CONTENT,
            FrameLayout.LayoutParams.WRAP_CONTENT,
            Gravity.TOP or Gravity.START
        ).apply { setMargins(dp(18), dp(18), dp(18), 0) })
        root.addView(menuButton("VOLTAR AO MENU", false) {
            surface.onPause()
            gameSurface = null
            showingGame = false
            showMainMenu()
        }, FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.WRAP_CONTENT,
            dp(42),
            Gravity.TOP or Gravity.END
        ).apply { setMargins(0, dp(18), dp(18), 0) })
        root.addView(label("CENA VOXEL DE TESTE  •  PRÉ-ALPHA", 9f, Color.WHITE, false).apply {
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
            .setTitle("Configurações")
            .setMessage(
                "Versão de teste\n\n" +
                    "Renderizador: OpenGL ES 3.0\n" +
                    "Plataforma: Android ARM64\n\n" +
                    "As opções gráficas e de controle serão adicionadas conforme o motor evoluir."
            )
            .setPositiveButton("OK", null)
            .show()
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

private class BookRenderer : GLSurfaceView.Renderer {
    private val bridge = NativeBridge()

    override fun onSurfaceCreated(
        gl: javax.microedition.khronos.opengles.GL10?,
        config: javax.microedition.khronos.egl.EGLConfig?
    ) {
        bridge.surfaceCreated()
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
    }
}

private class NativeBridge {
    fun surfaceCreated() {
        if (available) nativeSurfaceCreated()
    }

    fun surfaceChanged(width: Int, height: Int) {
        if (available) nativeSurfaceChanged(width, height)
    }

    fun drawFrame() {
        if (available) nativeDrawFrame()
    }

    private external fun nativeSurfaceCreated()
    private external fun nativeSurfaceChanged(width: Int, height: Int)
    private external fun nativeDrawFrame()

    companion object {
        val available: Boolean = runCatching {
            System.loadLibrary("book_android")
            true
        }.getOrDefault(false)
    }
}

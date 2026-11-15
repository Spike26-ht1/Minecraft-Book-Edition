package dev.bookedition.game

import android.app.Activity
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

class MainActivity : Activity() {
    private lateinit var gameSurface: GLSurfaceView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        requestedOrientation = android.content.pm.ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        hideSystemUi()

        val root = FrameLayout(this).apply {
            setBackgroundColor(Color.rgb(9, 14, 23))
        }

        gameSurface = GLSurfaceView(this).apply {
            setEGLContextClientVersion(3)
            preserveEGLContextOnPause = false
            setRenderer(BookRenderer())
            renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY
        }
        root.addView(
            gameSurface,
            FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
        )

        root.addView(createBrandPanel(), brandPanelLayoutParams())
        root.addView(createFooter(), footerLayoutParams())
        setContentView(root)
    }

    override fun onResume() {
        super.onResume()
        if (::gameSurface.isInitialized) gameSurface.onResume()
        hideSystemUi()
    }

    override fun onPause() {
        if (::gameSurface.isInitialized) gameSurface.onPause()
        super.onPause()
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (hasFocus) hideSystemUi()
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

    private fun createBrandPanel(): View {
        val panel = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(dp(14), dp(10), dp(18), dp(10))
            background = GradientDrawable().apply {
                setColor(Color.argb(208, 12, 19, 29))
                cornerRadius = dp(16).toFloat()
                setStroke(dp(1), Color.argb(90, 190, 211, 232))
            }
        }

        val mark = ImageView(this).apply {
            contentDescription = "Símbolo do Minecraft Book Edition"
            scaleType = ImageView.ScaleType.FIT_CENTER
            setImageBitmap(loadProjectMark())
        }
        panel.addView(mark, LinearLayout.LayoutParams(dp(58), dp(58)))

        val labels = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(dp(12), 0, 0, 0)
        }
        labels.addView(TextView(this).apply {
            text = "MINECRAFT BOOK EDITION"
            setTextColor(Color.WHITE)
            textSize = 15f
            typeface = Typeface.create("sans-serif-medium", Typeface.BOLD)
        })
        labels.addView(TextView(this).apply {
            text = if (NativeBridge.available) "BOOK64  •  TERRENO VOXEL  •  GLES 3.0" else "BOOK64 NATIVE LIBRARY NOT LOADED"
            setTextColor(Color.rgb(154, 190, 214))
            textSize = 10f
            letterSpacing = 0.08f
        })
        panel.addView(labels)
        return panel
    }

    private fun createFooter(): View {
        return TextView(this).apply {
            text = "PRÉ-ALPHA  •  TERRENO VOXEL DE TESTE  •  ANDROID ARM64"
            setTextColor(Color.argb(205, 229, 237, 247))
            textSize = 10f
            letterSpacing = 0.08f
            gravity = Gravity.CENTER
            setPadding(dp(12), dp(8), dp(12), dp(8))
            background = GradientDrawable().apply {
                setColor(Color.argb(155, 12, 19, 29))
                cornerRadius = dp(10).toFloat()
            }
        }
    }

    private fun loadProjectMark() = runCatching {
        val encoded = assets.open("book-edition-mark.png.base64")
            .bufferedReader(StandardCharsets.UTF_8)
            .use { it.readText() }
        val bytes = Base64.decode(encoded, Base64.DEFAULT)
        BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
    }.getOrNull()

    private fun brandPanelLayoutParams() =
        FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.WRAP_CONTENT,
            FrameLayout.LayoutParams.WRAP_CONTENT,
            Gravity.TOP or Gravity.START
        ).apply {
            setMargins(dp(18), dp(18), dp(18), 0)
        }

    private fun footerLayoutParams() =
        FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.WRAP_CONTENT,
            FrameLayout.LayoutParams.WRAP_CONTENT,
            Gravity.BOTTOM or Gravity.CENTER_HORIZONTAL
        ).apply {
            setMargins(dp(18), 0, dp(18), dp(16))
        }

    private fun dp(value: Int): Int =
        (value * resources.displayMetrics.density).toInt()
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

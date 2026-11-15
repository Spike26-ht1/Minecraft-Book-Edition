package dev.bookedition.game

import android.app.Activity
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.os.Build
import android.view.Gravity
import android.view.MotionEvent
import android.view.View
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.TextView
import java.util.Locale

/** Touch HUD for the first playable Book Edition camera prototype. */
internal class GameHud(
    private val activity: Activity,
    private val root: FrameLayout,
    private val bridge: NativeBridge,
    private val onMenu: () -> Unit
) {
    private val debugPanel = TextView(activity)
    private lateinit var controls: LinearLayout
    private lateinit var jumpButton: TextView
    private var debugVisible = false
    private var controlsVisible = true
    private var sensitivityLevel = 1
    private var sensitivity = 0.0035f
    private var forward = 0f
    private var strafe = 0f
    private var vertical = 0f
    private var fps = 0
    private var frameMs = 0.0
    private var nativeInfo = "Book64 initializing"
    private var fpsMode = "VSync"

    fun attach() {
        val lookSurface = View(activity).apply {
            setBackgroundColor(Color.TRANSPARENT)
            setOnTouchListener { _, event ->
                when (event.actionMasked) {
                    MotionEvent.ACTION_DOWN -> {
                        lastTouchX = event.x
                        lastTouchY = event.y
                        true
                    }
                    MotionEvent.ACTION_MOVE -> {
                        val dx = event.x - lastTouchX
                        val dy = event.y - lastTouchY
                        lastTouchX = event.x
                        lastTouchY = event.y
                        bridge.look(dx * sensitivity, -dy * sensitivity)
                        true
                    }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> true
                    else -> true
                }
            }
        }
        root.addView(lookSurface, FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.MATCH_PARENT,
            FrameLayout.LayoutParams.MATCH_PARENT
        ))

        val f3 = hudButton("F3").apply {
            setOnClickListener { toggleDebug() }
        }
        root.addView(f3, FrameLayout.LayoutParams(dp(54), dp(42), Gravity.TOP or Gravity.START).apply {
            setMargins(dp(16), dp(16), 0, 0)
        })

        val menu = hudButton("MENU").apply { setOnClickListener { onMenu() } }
        root.addView(menu, FrameLayout.LayoutParams(dp(76), dp(42), Gravity.TOP or Gravity.END).apply {
            setMargins(0, dp(16), dp(16), 0)
        })

        debugPanel.apply {
            textSize = 10f
            typeface = Typeface.MONOSPACE
            setTextColor(Color.WHITE)
            setPadding(dp(12), dp(10), dp(12), dp(10))
            background = rounded(Color.argb(205, 8, 15, 24), dp(10))
            visibility = View.GONE
        }
        root.addView(debugPanel, FrameLayout.LayoutParams(dp(310), FrameLayout.LayoutParams.WRAP_CONTENT, Gravity.TOP or Gravity.START).apply {
            setMargins(dp(16), dp(66), 0, 0)
        })

        controls = LinearLayout(activity).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER
        }
        controls.addView(controlRow("▲") { holdButton(it, 1f, 0f) })
        val middle = LinearLayout(activity).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER
        }
        middle.addView(directionButton("◀", -1f))
        middle.addView(directionButton("▶", 1f))
        controls.addView(middle)
        controls.addView(controlRow("▼") { holdButton(it, -1f, 0f) })
        root.addView(controls, FrameLayout.LayoutParams(
            FrameLayout.LayoutParams.WRAP_CONTENT,
            FrameLayout.LayoutParams.WRAP_CONTENT,
            Gravity.BOTTOM or Gravity.START
        ).apply { setMargins(dp(18), 0, 0, dp(18)) })

        jumpButton = hudButton("SUBIR").apply {
            setOnTouchListener { _, event ->
                when (event.actionMasked) {
                    MotionEvent.ACTION_DOWN -> { vertical = 1f; sendMovement(); true }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> { vertical = 0f; sendMovement(); true }
                    else -> true
                }
            }
        }
        root.addView(jumpButton, FrameLayout.LayoutParams(dp(92), dp(58), Gravity.BOTTOM or Gravity.END).apply {
            setMargins(0, 0, dp(24), dp(28))
        })
    }

    private var lastTouchX = 0f
    private var lastTouchY = 0f

    fun updateFrame(currentFps: Int, currentFrameMs: Double, currentFpsMode: String) {
        fps = currentFps
        frameMs = currentFrameMs
        fpsMode = currentFpsMode
        if (!debugVisible) return
        nativeInfo = bridge.debugInfo()
        val metrics = activity.resources.displayMetrics
        val runtime = Runtime.getRuntime()
        val usedMb = (runtime.totalMemory() - runtime.freeMemory()) / (1024L * 1024L)
        val totalMb = runtime.totalMemory() / (1024L * 1024L)
        val report = String.format(
            Locale.US,
            "BOOK EDITION DEBUG\nFPS: %d  |  Frame CPU: %.2f ms\nFPS mode: %s\nRenderer: Book64 / OpenGL ES 3.0\nDevice: %s %s\nAndroid: %s (API %d)\nDisplay: %d × %d  |  Density: %.2f\nHeap: %d / %d MB\nWorld: local voxel test scene\nChunk system: 16 × 16 × 16 section\n%s",
            fps, frameMs, fpsMode, Build.MANUFACTURER, Build.MODEL, Build.VERSION.RELEASE,
            Build.VERSION.SDK_INT, metrics.widthPixels, metrics.heightPixels, metrics.density,
            usedMb, totalMb, nativeInfo
        )
        debugPanel.post { debugPanel.text = report }
    }

    fun toggleDebug() {
        debugVisible = !debugVisible
        debugPanel.visibility = if (debugVisible) View.VISIBLE else View.GONE
        if (debugVisible) updateFrame(fps, frameMs, fpsMode)
    }

    fun toggleControls(): Boolean {
        controlsVisible = !controlsVisible
        controls.visibility = if (controlsVisible) View.VISIBLE else View.GONE
        jumpButton.visibility = if (controlsVisible) View.VISIBLE else View.GONE
        return controlsVisible
    }

    fun cycleSensitivity(): String {
        sensitivityLevel = (sensitivityLevel + 1) % 3
        sensitivity = when (sensitivityLevel) {
            0 -> 0.0020f
            1 -> 0.0035f
            else -> 0.0050f
        }
        return when (sensitivityLevel) {
            0 -> "Baixa"
            1 -> "Média"
            else -> "Alta"
        }
    }

    private fun controlRow(symbol: String, onTouch: (MotionEvent) -> Boolean): View {
        val row = LinearLayout(activity).apply { gravity = Gravity.CENTER }
        val button = hudButton(symbol).apply {
            setOnTouchListener { _, event -> onTouch(event) }
        }
        row.addView(button, LinearLayout.LayoutParams(dp(52), dp(48)).apply {
            setMargins(dp(3), dp(3), dp(3), dp(3))
        })
        return row
    }

    private fun directionButton(symbol: String, side: Float): View =
        hudButton(symbol).apply {
            setOnTouchListener { _, event ->
                when (event.actionMasked) {
                    MotionEvent.ACTION_DOWN -> { strafe = side; sendMovement(); true }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> { strafe = 0f; sendMovement(); true }
                    else -> true
                }
            }
            layoutParams = LinearLayout.LayoutParams(dp(52), dp(48)).apply {
                setMargins(dp(3), dp(3), dp(3), dp(3))
            }
        }

    private fun holdButton(event: MotionEvent, forwardValue: Float, verticalValue: Float): Boolean {
        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                forward = forwardValue
                vertical = verticalValue
                sendMovement()
            }
            MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                forward = 0f
                vertical = 0f
                sendMovement()
            }
        }
        return true
    }

    private fun sendMovement() = bridge.setMovement(forward, strafe, vertical)

    private fun hudButton(text: String) = TextView(activity).apply {
        this.text = text
        gravity = Gravity.CENTER
        textSize = 13f
        typeface = Typeface.create("sans-serif-medium", Typeface.BOLD)
        setTextColor(Color.WHITE)
        background = rounded(Color.argb(205, 21, 37, 51), dp(12))
        isClickable = true
        isFocusable = true
    }

    private fun rounded(color: Int, radius: Int) = GradientDrawable().apply {
        setColor(color)
        cornerRadius = radius.toFloat()
        setStroke(dp(1), Color.argb(90, 220, 240, 255))
    }

    private fun dp(value: Int): Int = (value * activity.resources.displayMetrics.density).toInt()
}

package dev.bookedition.launcher

import android.content.Context
import android.util.Log
import java.io.File
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

internal object LauncherLogger {
    private const val TAG = "BookEdition"
    private const val LOG_FILE = "launcher.log"
    private val lock = Any()

    fun info(context: Context, message: String) {
        write(context, "INFO", message)
    }

    fun debug(context: Context, message: String) {
        if (LauncherSettings.areDetailedLogsEnabled(context)) {
            write(context, "DEBUG", message)
        }
    }

    fun read(context: Context): String {
        val file = logFile(context)
        return runCatching { if (file.isFile) file.readText(Charsets.UTF_8) else "" }
            .getOrDefault("")
    }

    private fun write(context: Context, level: String, message: String) {
        runCatching {
            val file = logFile(context)
            val directory = file.parentFile ?: error("Diretório de logs inválido.")
            if (!directory.exists() && !directory.mkdirs()) {
                error("Não foi possível criar o diretório de logs.")
            }
            val timestamp = SimpleDateFormat("yyyy-MM-dd HH:mm:ss", Locale.US).format(Date())
            synchronized(lock) {
                file.appendText("$timestamp [$level] $message\n", Charsets.UTF_8)
            }
        }.onFailure {
            Log.e(TAG, "Não foi possível gravar o log local.", it)
        }
    }

    private fun logFile(context: Context): File =
        File(context.filesDir, "book-edition/launcher/logs/$LOG_FILE")
}

package dev.bookedition.launcher

import android.content.Context

internal object LauncherSettings {
    private const val PREFERENCES = "book_edition_preferences"
    private const val KEY_DARK_THEME = "dark_theme"
    private const val KEY_DETAILED_LOGS = "detailed_logs"

    fun isDarkTheme(context: Context): Boolean =
        preferences(context).getBoolean(KEY_DARK_THEME, true)

    fun setDarkTheme(context: Context, enabled: Boolean) {
        preferences(context).edit().putBoolean(KEY_DARK_THEME, enabled).apply()
    }

    fun areDetailedLogsEnabled(context: Context): Boolean =
        preferences(context).getBoolean(KEY_DETAILED_LOGS, false)

    fun setDetailedLogsEnabled(context: Context, enabled: Boolean) {
        preferences(context).edit().putBoolean(KEY_DETAILED_LOGS, enabled).apply()
    }

    private fun preferences(context: Context) =
        context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)
}

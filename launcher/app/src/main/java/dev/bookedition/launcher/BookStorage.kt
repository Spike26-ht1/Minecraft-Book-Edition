package dev.bookedition.launcher

import android.content.Context
import java.io.File
import java.io.IOException

internal object StorageLayout {
    val requiredDirectories = listOf(
        "launcher/settings",
        "launcher/logs",
        "launcher/versions",
        "minecraft/assets",
        "minecraft/libraries",
        "minecraft/resources",
        "minecraft/worlds",
        "minecraft/cache"
    )

    fun createDirectories(root: File) {
        if (!root.exists() && !root.mkdirs()) {
            throw IOException("Não foi possível criar o diretório de dados.")
        }
        if (!root.isDirectory) {
            throw IOException("O caminho de dados não é um diretório.")
        }

        requiredDirectories.forEach { relativePath ->
            val directory = File(root, relativePath)
            if (!directory.exists() && !directory.mkdirs()) {
                throw IOException("Não foi possível criar: $relativePath")
            }
            if (!directory.isDirectory) {
                throw IOException("O caminho esperado não é um diretório: $relativePath")
            }
        }
    }
}

internal object BookStorage {
    fun rootPath(context: Context): File = File(context.filesDir, "book-edition")

    fun externalMediaRoot(context: Context): File? =
        context.getExternalMediaDirs().firstOrNull()?.let { File(it, "book-edition") }

    fun initialize(context: Context): File {
        val root = rootPath(context)
        StorageLayout.createDirectories(root)
        return root
    }

    fun initializeExternalMedia(context: Context): File? {
        val root = externalMediaRoot(context) ?: return null
        return runCatching {
            val folders = listOf("resource-packs", "worlds", "exports", "logs", "cache")
            if (!root.exists() && !root.mkdirs()) error("Não foi possível criar Android/media.")
            folders.forEach { name ->
                val folder = File(root, name)
                if (!folder.exists() && !folder.mkdirs()) error("Não foi possível criar $name.")
                if (!folder.isDirectory) error("$name não é um diretório.")
            }
            root
        }.getOrNull()
    }
}

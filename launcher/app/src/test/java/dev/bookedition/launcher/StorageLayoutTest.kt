package dev.bookedition.launcher

import java.io.File
import java.nio.file.Files
import org.junit.Assert.assertTrue
import org.junit.Test

class StorageLayoutTest {
    @Test
    fun createsEveryRequiredPrivateDirectory() {
        val root = Files.createTempDirectory("book-edition-storage-test").toFile()
        try {
            StorageLayout.createDirectories(root)

            StorageLayout.requiredDirectories.forEach { relativePath ->
                assertTrue(
                    "Expected directory to exist: $relativePath",
                    File(root, relativePath).isDirectory
                )
            }
        } finally {
            root.deleteRecursively()
        }
    }
}

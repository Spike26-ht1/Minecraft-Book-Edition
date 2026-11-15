package dev.bookedition.game

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Rect
import android.net.Uri
import org.json.JSONArray
import org.json.JSONObject
import java.io.ByteArrayOutputStream
import java.io.File
import java.util.zip.ZipInputStream
import kotlin.math.max

/** Safe, local-only importer for user supplied Bedrock resource packs. */
internal object ResourcePackImporter {
    data class Report(val name: String, val loaded: Int, val missing: Int, val width: Int, val height: Int)
    private const val MAX_UNPACKED = 48L * 1024L * 1024L
    private const val MAX_ENTRIES = 12000
    private const val ATLAS_WIDTH = 2048
    private const val MAX_ATLAS_HEIGHT = 4096
    private const val PAD = 2

    fun import(context: Context, uri: Uri): Report {
        val files = readZip(context, uri)
        val manifestEntry = files.entries.firstOrNull { it.key.substringAfterLast('/') == "manifest.json" }
            ?: error("manifest.json não encontrado. Selecione um .mcpack ou ZIP Bedrock.")
        val manifest = JSONObject(manifestEntry.value.toString(Charsets.UTF_8))
        val header = manifest.optJSONObject("header") ?: error("Manifest sem header.")
        val packName = header.optString("name", "Resource Pack")
        val terrainEntry = files.entries.firstOrNull { it.key.endsWith("textures/terrain_texture.json", true) }
            ?: error("textures/terrain_texture.json não encontrado.")
        val terrain = JSONObject(terrainEntry.value.toString(Charsets.UTF_8))
        val data = terrain.optJSONObject("texture_data") ?: error("texture_data não encontrado.")
        val aliases = JSONObject()
        val paths = LinkedHashSet<String>()
        val keys = data.keys()
        while (keys.hasNext()) {
            val key = keys.next()
            val found = ArrayList<String>()
            collect(data.opt(key), found)
            if (found.isNotEmpty()) {
                aliases.put(key, found.first())
                paths.addAll(found)
            }
        }
        if (paths.isEmpty()) error("O atlas não contém referências de textura.")
        val bitmaps = LinkedHashMap<String, Bitmap>()
        var missing = 0
        for (path in paths) {
            val bytes = resolve(files, path, terrainEntry.key)
            if (bytes == null) {
                missing++
                continue
            }
            val bitmap = BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
            if (bitmap == null || bitmap.width !in 1..1024 || bitmap.height !in 1..1024) {
                bitmap?.recycle()
                missing++
            } else bitmaps[path] = bitmap
        }
        if (bitmaps.isEmpty()) error("Nenhuma textura referenciada pôde ser decodificada.")
        val placements = LinkedHashMap<String, Rect>()
        var x = PAD
        var y = PAD
        var rowHeight = 0
        for ((path, bitmap) in bitmaps) {
            val w = bitmap.width + PAD * 2
            val h = bitmap.height + PAD * 2
            if (w > ATLAS_WIDTH) error("Textura maior que a largura suportada.")
            if (x + w > ATLAS_WIDTH) {
                x = PAD
                y += rowHeight
                rowHeight = 0
            }
            if (y + h > MAX_ATLAS_HEIGHT) error("Atlas excede 2048×4096. Este pack precisa ser dividido em páginas.")
            placements[path] = Rect(x, y, x + bitmap.width, y + bitmap.height)
            x += w
            rowHeight = max(rowHeight, h)
        }
        val atlasHeight = nextPowerOfTwo(max(1, y + rowHeight))
        val atlas = Bitmap.createBitmap(ATLAS_WIDTH, atlasHeight, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(atlas)
        val paint = Paint(Paint.FILTER_BITMAP_FLAG)
        for ((path, bitmap) in bitmaps) {
            val r = placements.getValue(path)
            canvas.drawBitmap(bitmap, null, r, paint)
            canvas.drawBitmap(atlas, Rect(r.left, r.top, r.right, r.top + 1),
                Rect(r.left, r.top - 1, r.right, r.top), null)
            canvas.drawBitmap(atlas, Rect(r.left, r.bottom - 1, r.right, r.bottom),
                Rect(r.left, r.bottom, r.right, r.bottom + 1), null)
        }
        val folder = File(context.filesDir, "resource-packs/active").apply { mkdirs() }
        File(folder, "terrain-atlas.png").outputStream().use { atlas.compress(Bitmap.CompressFormat.PNG, 100, it) }
        val uv = JSONObject()
        for ((path, r) in placements) {
            uv.put(path, JSONArray().put(r.left.toDouble() / ATLAS_WIDTH)
                .put(r.top.toDouble() / atlasHeight).put(r.right.toDouble() / ATLAS_WIDTH)
                .put(r.bottom.toDouble() / atlasHeight))
        }
        val index = JSONObject().put("pack_name", packName).put("manifest_header", header)
            .put("atlas_width", ATLAS_WIDTH).put("atlas_height", atlasHeight)
            .put("loaded_textures", bitmaps.size).put("missing_textures", missing)
            .put("texture_data", aliases).put("uv", uv)
        File(folder, "atlas-index.json").writeText(index.toString())
        bitmaps.values.forEach { it.recycle() }
        atlas.recycle()
        context.getSharedPreferences("book_resource_packs", Context.MODE_PRIVATE).edit()
            .putString("active_pack_name", packName)
            .putString("active_pack_summary", "$packName • \${bitmaps.size} texturas • $missing ausentes")
            .putInt("active_pack_texture_count", bitmaps.size)
            .putInt("active_pack_missing_count", missing).apply()
        return Report(packName, bitmaps.size, missing, ATLAS_WIDTH, atlasHeight)
    }

    private fun readZip(context: Context, uri: Uri): LinkedHashMap<String, ByteArray> {
        val result = LinkedHashMap<String, ByteArray>()
        var total = 0L
        var count = 0
        val stream = context.contentResolver.openInputStream(uri) ?: error("Não foi possível abrir o arquivo.")
        ZipInputStream(stream.buffered()).use { zip ->
            while (true) {
                val entry = zip.nextEntry ?: break
                count++
                if (count > MAX_ENTRIES) error("Arquivo contém entradas demais.")
                val name = entry.name.replace('\\', '/')
                if (!entry.isDirectory && !name.startsWith("/") && name.split('/').none { it == ".." } &&
                    (name.endsWith(".json", true) || name.endsWith(".png", true))) {
                    val out = ByteArrayOutputStream()
                    val buffer = ByteArray(8192)
                    while (true) {
                        val n = zip.read(buffer)
                        if (n < 0) break
                        total += n
                        if (total > MAX_UNPACKED) error("Conteúdo descompactado excede 48 MB.")
                        out.write(buffer, 0, n)
                    }
                    result[name] = out.toByteArray()
                }
                zip.closeEntry()
            }
        }
        return result
    }

    private fun collect(value: Any?, output: MutableList<String>) {
        when (value) {
            is String -> normalize(value)?.let(output::add)
            is JSONArray -> for (i in 0 until value.length()) collect(value.opt(i), output)
            is JSONObject -> {
                val path = value.optString("path", "")
                if (path.isNotBlank()) normalize(path)?.let(output::add)
                value.opt("textures")?.let { collect(it, output) }
            }
        }
    }

    private fun normalize(value: String): String? {
        val path = value.trim().replace('\\', '/').removePrefix("/")
        if (path.isBlank() || path.split('/').any { it == ".." }) return null
        return if (path.endsWith(".png", true)) path else "$path.png"
    }

    private fun resolve(files: Map<String, ByteArray>, path: String, terrain: String): ByteArray? {
        files[path]?.let { return it }
        val root = terrain.substringBeforeLast("textures/terrain_texture.json")
        files[root + path]?.let { return it }
        return files.entries.firstOrNull { it.key.endsWith("/$path", true) }?.value
    }

    private fun nextPowerOfTwo(value: Int): Int {
        var result = 1
        while (result < value && result < MAX_ATLAS_HEIGHT) result *= 2
        return result
    }
}

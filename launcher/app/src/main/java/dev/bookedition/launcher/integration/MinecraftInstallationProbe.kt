package dev.bookedition.launcher.integration

import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import java.security.MessageDigest

/**
 * Read-only probe for the official Minecraft for Android package.
 *
 * This only detects an installed package and reports its signing-certificate
 * fingerprint. It deliberately does NOT claim that installation proves purchase
 * or a valid license. A trusted certificate allow-list must be maintained from
 * independently verified official release metadata before signer validation can
 * be treated as an authenticity check.
 */
object MinecraftInstallationProbe {
    const val OFFICIAL_PACKAGE = "com.mojang.minecraftpe"

    data class Result(
        val installed: Boolean,
        val packageName: String = OFFICIAL_PACKAGE,
        val versionName: String? = null,
        val versionCode: Long? = null,
        val signerSha256: List<String> = emptyList(),
        val statusMessage: String
    )

    @Suppress("DEPRECATION")
    fun inspect(context: Context): Result {
        val pm = context.packageManager
        val info = try {
            if (Build.VERSION.SDK_INT >= 33) {
                pm.getPackageInfo(
                    OFFICIAL_PACKAGE,
                    PackageManager.PackageInfoFlags.of(
                        PackageManager.GET_SIGNING_CERTIFICATES.toLong()
                    )
                )
            } else {
                pm.getPackageInfo(
                    OFFICIAL_PACKAGE,
                    PackageManager.GET_SIGNING_CERTIFICATES
                )
            }
        } catch (_: PackageManager.NameNotFoundException) {
            return Result(
                installed = false,
                statusMessage = "Minecraft Bedrock não foi encontrado neste aparelho."
            )
        } catch (_: SecurityException) {
            return Result(
                installed = false,
                statusMessage = "O Android não permitiu consultar o pacote Minecraft."
            )
        }

        val signatures = if (Build.VERSION.SDK_INT >= 28) {
            info.signingInfo?.apkContentsSigners?.toList().orEmpty()
        } else {
            @Suppress("DEPRECATION")
            info.signatures?.toList().orEmpty()
        }

        val fingerprints = signatures.map { signature ->
            MessageDigest.getInstance("SHA-256")
                .digest(signature.toByteArray())
                .joinToString("") { byte -> "%02X".format(byte) }
        }

        val versionCode = if (Build.VERSION.SDK_INT >= 28) {
            info.longVersionCode
        } else {
            @Suppress("DEPRECATION")
            info.versionCode.toLong()
        }

        return Result(
            installed = true,
            versionName = info.versionName,
            versionCode = versionCode,
            signerSha256 = fingerprints,
            statusMessage = "Minecraft encontrado. A presença do aplicativo não comprova a licença."
        )
    }
}

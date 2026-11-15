package dev.bookedition.launcher

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import java.io.File

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        runCatching { BookStorage.initialize(this) }
            .onFailure { android.util.Log.e("BookEdition", "Falha ao preparar armazenamento privado", it) }
        LauncherLogger.info(this, "Launcher iniciado • versão ${BuildConfig.VERSION_NAME}")

        setContent {
            BookEditionApp()
        }
    }
}

private enum class LauncherPage(val label: String, val symbol: String) {
    HOME("Início", "⌂"),
    VERSIONS("Versões", "▤"),
    SETTINGS("Ajustes", "⚙"),
    LOGS("Logs", "≡")
}

@Composable
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun BookEditionApp() {
    val context = LocalContext.current
    var selectedPage by remember { mutableStateOf(LauncherPage.HOME) }
    var darkTheme by remember { mutableStateOf(LauncherSettings.isDarkTheme(context)) }
    var detailedLogs by remember { mutableStateOf(LauncherSettings.areDetailedLogsEnabled(context)) }
    var logContents by remember { mutableStateOf(LauncherLogger.read(context)) }
    var exportFeedback by remember { mutableStateOf("") }

    val exportLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.CreateDocument("text/plain")
    ) { uri ->
        if (uri == null) {
            exportFeedback = "Exportação cancelada."
        } else {
            val result = runCatching {
                val output = context.contentResolver.openOutputStream(uri)
                    ?: error("Não foi possível abrir o arquivo de destino.")
                output.bufferedWriter(Charsets.UTF_8).use { it.write(LauncherLogger.read(context)) }
            }
            exportFeedback = if (result.isSuccess) {
                "Relatório exportado."
            } else {
                "Não foi possível exportar o relatório."
            }
        }
    }

    BookEditionTheme(darkTheme = darkTheme) {
        Scaffold(
            topBar = {
                CenterAlignedTopAppBar(
                    title = {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Text(
                                text = "Minecraft Book Edition",
                                style = MaterialTheme.typography.titleMedium,
                                fontWeight = FontWeight.SemiBold
                            )
                            Text(
                                text = selectedPage.label,
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                )
            },
            bottomBar = {
                NavigationBar {
                    LauncherPage.values().forEach { page ->
                        NavigationBarItem(
                            selected = selectedPage == page,
                            onClick = {
                                selectedPage = page
                                LauncherLogger.debug(context, "Aba aberta: ${page.label}")
                                if (page == LauncherPage.LOGS) {
                                    logContents = LauncherLogger.read(context)
                                }
                            },
                            icon = { Text(page.symbol, fontSize = 18.sp) },
                            label = { Text(page.label) }
                        )
                    }
                }
            }
        ) { contentPadding ->
            Surface(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(contentPadding),
                color = MaterialTheme.colorScheme.background
            ) {
                when (selectedPage) {
                    LauncherPage.HOME -> HomeScreen(
                        onOpenVersions = { selectedPage = LauncherPage.VERSIONS },
                        onOpenLogs = { selectedPage = LauncherPage.LOGS }
                    )
                    LauncherPage.VERSIONS -> VersionsScreen()
                    LauncherPage.SETTINGS -> SettingsScreen(
                        darkTheme = darkTheme,
                        onDarkThemeChange = {
                            darkTheme = it
                            LauncherSettings.setDarkTheme(context, it)
                            LauncherLogger.info(context, "Tema alterado: ${if (it) "escuro" else "claro"}")
                        },
                        detailedLogs = detailedLogs,
                        onDetailedLogsChange = {
                            detailedLogs = it
                            LauncherSettings.setDetailedLogsEnabled(context, it)
                            LauncherLogger.info(context, "Registros detalhados: ${if (it) "ativados" else "desativados"}")
                        },
                        storageRoot = BookStorage.rootPath(context)
                    )
                    LauncherPage.LOGS -> LogsScreen(
                        logs = logContents,
                        feedback = exportFeedback,
                        onRefresh = {
                            logContents = LauncherLogger.read(context)
                            exportFeedback = "Logs atualizados."
                        },
                        onExport = {
                            exportFeedback = ""
                            exportLauncher.launch("book-edition-logs.txt")
                        }
                    )
                }
            }
        }
    }
}

@Composable
private fun HomeScreen(
    onOpenVersions: () -> Unit,
    onOpenLogs: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(20.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = "UM NOVO CAPÍTULO",
            color = MaterialTheme.colorScheme.primary,
            style = MaterialTheme.typography.labelLarge,
            letterSpacing = 2.sp
        )
        Text(
            text = "Seu Minecraft.\nUma base aberta.",
            style = MaterialTheme.typography.headlineLarge,
            fontWeight = FontWeight.Bold,
            lineHeight = 38.sp
        )
        Text(
            text = "Um launcher em desenvolvimento, criado para ser claro, modular e fácil de entender.",
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )

        Card(
            modifier = Modifier.fillMaxWidth(),
            shape = RoundedCornerShape(22.dp),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
        ) {
            Column(modifier = Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text("STATUS DO PROJETO", style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.primary)
                    Spacer(Modifier.weight(1f))
                    Surface(
                        color = MaterialTheme.colorScheme.secondaryContainer,
                        shape = RoundedCornerShape(50)
                    ) {
                        Text(
                            "EM DESENVOLVIMENTO",
                            modifier = Modifier.padding(horizontal = 10.dp, vertical = 5.dp),
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSecondaryContainer
                        )
                    }
                }
                Text("Launcher pronto para testes iniciais", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.SemiBold)
                Text(
                    "A interface e o armazenamento local estão disponíveis. O cliente Bedrock ainda não foi integrado.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }

        Button(
            onClick = { },
            enabled = false,
            modifier = Modifier.fillMaxWidth().height(54.dp),
            shape = RoundedCornerShape(16.dp)
        ) {
            Text("Jogar • indisponível nesta versão")
        }
        Text(
            "O botão será ativado quando existir uma implementação de cliente compatível.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )

        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            ActionCard(
                title = "Versões",
                description = "Estado do runtime",
                symbol = "▤",
                modifier = Modifier.weight(1f),
                onClick = onOpenVersions
            )
            ActionCard(
                title = "Diagnóstico",
                description = "Consultar registros",
                symbol = "≡",
                modifier = Modifier.weight(1f),
                onClick = onOpenLogs
            )
        }

        Text(
            "BUILD ${BuildConfig.VERSION_NAME}",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            letterSpacing = 1.sp
        )
    }
}

@Composable
private fun ActionCard(
    title: String,
    description: String,
    symbol: String,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    Card(
        modifier = modifier,
        onClick = onClick,
        shape = RoundedCornerShape(18.dp),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Text(symbol, color = MaterialTheme.colorScheme.primary, fontSize = 24.sp)
            Text(title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
            Text(description, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
private fun VersionsScreen() {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(20.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text("Gerenciador de versões", style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.Bold)
        Text(
            "Esta área vai organizar as versões do cliente e seus arquivos. Nenhum runtime é instalado ou presumido nesta build.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)) {
            Column(Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("CLIENTE BEDROCK", style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.primary)
                Text("Não configurado", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.SemiBold)
                Text(
                    "A integração do cliente ainda depende da decisão de arquitetura e não faz parte desta versão de testes.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
        Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)) {
            Column(Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Text("LAUNCHER", style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.primary)
                Text(BuildConfig.VERSION_NAME, style = MaterialTheme.typography.titleMedium)
                Text("Versão instalada do aplicativo Book Edition.", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}

@Composable
private fun SettingsScreen(
    darkTheme: Boolean,
    onDarkThemeChange: (Boolean) -> Unit,
    detailedLogs: Boolean,
    onDetailedLogsChange: (Boolean) -> Unit,
    storageRoot: File
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(20.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text("Preferências", style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.Bold)
        Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)) {
            Column(Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                SettingSwitch(
                    title = "Tema escuro",
                    description = "Usar a aparência escura do launcher.",
                    checked = darkTheme,
                    onCheckedChange = onDarkThemeChange
                )
                SettingSwitch(
                    title = "Registros detalhados",
                    description = "Adicionar eventos de navegação e diagnóstico aos logs locais.",
                    checked = detailedLogs,
                    onCheckedChange = onDetailedLogsChange
                )
            }
        }
        Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)) {
            Column(Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("ARMAZENAMENTO PRIVADO", style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.primary)
                Text("Diretório de dados", style = MaterialTheme.typography.titleMedium)
                Text(
                    storageRoot.absolutePath,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Text(
                    "Os dados ficam no espaço privado do aplicativo. A desinstalação do app pode removê-los.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
        Text(
            "Minecraft Book Edition • ${BuildConfig.VERSION_NAME}",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

@Composable
private fun SettingSwitch(
    title: String,
    description: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f).padding(end = 12.dp)) {
            Text(title, style = MaterialTheme.typography.titleMedium)
            Text(description, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Switch(checked = checked, onCheckedChange = onCheckedChange)
    }
}

@Composable
private fun LogsScreen(
    logs: String,
    feedback: String,
    onRefresh: () -> Unit,
    onExport: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(20.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp)
    ) {
        Text("Diagnóstico", style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.Bold)
        Text(
            "Os registros são locais. A exportação só acontece quando você escolhe um destino.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Button(onClick = onRefresh, modifier = Modifier.weight(1f)) {
                Text("Atualizar")
            }
            Button(onClick = onExport, modifier = Modifier.weight(1f)) {
                Text("Exportar")
            }
        }
        if (feedback.isNotBlank()) {
            Text(feedback, color = MaterialTheme.colorScheme.primary, style = MaterialTheme.typography.bodySmall)
        }
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
        ) {
            Text(
                text = logs.takeLast(10000).ifBlank { "Nenhum registro disponível ainda." },
                modifier = Modifier.padding(16.dp),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
        Text(
            "O relatório não é enviado automaticamente para nenhum servidor.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

@Composable
private fun BookEditionTheme(
    darkTheme: Boolean,
    content: @Composable () -> Unit
) {
    val colors = if (darkTheme) {
        darkColorScheme(
            primary = Color(0xFFA9D8B7),
            onPrimary = Color(0xFF102016),
            secondary = Color(0xFFB8CDBD),
            background = Color(0xFF0D1511),
            surface = Color(0xFF121D17),
            surfaceVariant = Color(0xFF1B2921),
            onSurface = Color(0xFFE4EEE6),
            onSurfaceVariant = Color(0xFFB2C2B6)
        )
    } else {
        lightColorScheme(
            primary = Color(0xFF245B3B),
            onPrimary = Color.White,
            secondary = Color(0xFF496653),
            background = Color(0xFFF4F7F3),
            surface = Color(0xFFFFFFFF),
            surfaceVariant = Color(0xFFE6EEE7),
            onSurface = Color(0xFF17221A),
            onSurfaceVariant = Color(0xFF526257)
        )
    }

    MaterialTheme(colorScheme = colors, content = content)
}

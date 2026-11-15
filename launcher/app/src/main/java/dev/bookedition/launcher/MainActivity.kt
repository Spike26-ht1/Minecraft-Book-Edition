package dev.bookedition.launcher

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent { BookLauncherTheme { LauncherHome() } }
    }
}

@Composable
private fun BookLauncherTheme(content: @Composable () -> Unit) {
    MaterialTheme(colorScheme = darkColorScheme(), content = content)
}

@Composable
private fun LauncherHome() {
    Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        Column(
            modifier = Modifier.fillMaxSize().padding(24.dp),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text("Minecraft Book Edition", style = MaterialTheme.typography.headlineMedium)
            Spacer(Modifier.height(8.dp))
            Text("Launcher • versão 0.1.0-dev.1", style = MaterialTheme.typography.bodyMedium)
            Spacer(Modifier.height(32.dp))
            Button(onClick = { /* Cliente será integrado em etapa posterior. */ }) {
                Text("Jogar")
            }
            Spacer(Modifier.height(12.dp))
            Text("O cliente Bedrock ainda não está integrado.")
        }
    }
}

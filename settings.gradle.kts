pluginManagement {
    repositories { google(); mavenCentral(); gradlePluginPortal() }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories { google(); mavenCentral() }
}
rootProject.name = "MinecraftBookEdition"
include(":launcher:app", ":game-runtime")
project(":game-runtime").projectDir = file("game/android/app")

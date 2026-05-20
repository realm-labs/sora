plugins {
    kotlin("jvm") version "1.6.21"
    application
}

kotlin {
    sourceSets["main"].kotlin.srcDirs("../generated/kotlin", "src/main/kotlin")
}

application {
    mainClass.set("game_config_showcase.smoke.MainKt")
}

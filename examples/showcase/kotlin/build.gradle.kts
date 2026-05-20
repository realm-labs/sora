plugins {
    kotlin("jvm") version "2.3.20"
    application
}

kotlin {
    jvmToolchain(21)
    sourceSets["main"].kotlin.srcDirs("src/generated/kotlin", "src/main/kotlin")
}

application {
    mainClass.set("com.sora.showcase.MainKt")
}

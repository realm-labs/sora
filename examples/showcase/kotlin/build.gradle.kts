plugins {
    kotlin("jvm") version "2.3.20"
    application
}

kotlin {
    jvmToolchain(21)
    sourceSets["main"].kotlin.srcDirs("src/generated/kotlin", "src/main/kotlin")
}

dependencies {
    implementation("com.fasterxml.jackson.core:jackson-databind:2.21.3")
    implementation("com.fasterxml.jackson.dataformat:jackson-dataformat-cbor:2.21.3")
    implementation("com.google.protobuf:protobuf-java:4.34.1")
}

application {
    mainClass.set("com.sora.showcase.MainKt")
}

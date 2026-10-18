plugins {
    kotlin("jvm") version "2.3.21"
    application
}

kotlin {
    jvmToolchain(21)
    sourceSets["main"].kotlin.srcDirs("src/generated/kotlin", "src/main/kotlin")
}

dependencies {
    implementation("com.fasterxml.jackson.core:jackson-databind:2.22.4")
    implementation("com.fasterxml.jackson.dataformat:jackson-dataformat-cbor:2.22.4")
    implementation("com.google.protobuf:protobuf-java:4.36.2")
    implementation("com.github.luben:zstd-jni:1.5.7-23")
}

application {
    mainClass.set("com.sora.showcase.MainKt")
}

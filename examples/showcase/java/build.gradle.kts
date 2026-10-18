plugins {
    application
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

dependencies {
    implementation("com.fasterxml.jackson.core:jackson-databind:2.22.4")
    implementation("com.fasterxml.jackson.dataformat:jackson-dataformat-cbor:2.22.4")
    implementation("com.google.protobuf:protobuf-java:4.36.2")
    implementation("com.github.luben:zstd-jni:1.5.7-23")
}

sourceSets {
    main {
        java.srcDirs("src/generated/java", "src/main/java")
    }
}

application {
    mainClass.set("com.sora.showcase.Main")
}

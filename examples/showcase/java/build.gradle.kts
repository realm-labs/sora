plugins {
    application
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

dependencies {
    implementation("com.fasterxml.jackson.core:jackson-databind:2.21.3")
    implementation("com.fasterxml.jackson.dataformat:jackson-dataformat-cbor:2.21.3")
}

sourceSets {
    main {
        java.srcDirs("src/generated/java", "src/main/java")
    }
}

application {
    mainClass.set("com.sora.showcase.Main")
}

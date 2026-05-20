plugins {
    application
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

sourceSets {
    main {
        java.srcDirs("src/generated/java", "src/main/java")
    }
}

application {
    mainClass.set("com.sora.showcase.Main")
}

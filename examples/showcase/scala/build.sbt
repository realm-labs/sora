ThisBuild / scalaVersion := "3.3.3"

lazy val root = (project in file("."))
  .settings(
    name := "sora-showcase-scala",
    Compile / unmanagedSourceDirectories := Seq(
      baseDirectory.value / "src" / "generated" / "scala",
      baseDirectory.value / "src" / "main" / "scala",
    ),
    libraryDependencies += "com.github.luben" % "zstd-jni" % "1.5.7-5",
    run / fork := true,
  )

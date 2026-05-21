ThisBuild / scalaVersion := "3.3.3"

lazy val root = (project in file("."))
  .settings(
    name := "sora-showcase-scala",
    Compile / unmanagedSourceDirectories := Seq(
      baseDirectory.value / "src" / "generated" / "scala",
      baseDirectory.value / "src" / "main" / "scala",
    ),
    run / fork := true,
  )

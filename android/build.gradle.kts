// The app that includes this plugin decides the Kotlin version: Tauri 2 projects made
// before 2026 use Kotlin 1.9, new ones Kotlin 2.x. Kotlin 2 needs the Compose Compiler
// Gradle plugin (same version as Kotlin); Kotlin 1.9 needs the old compiler artifact.
buildscript {
    // Tauri puts the Kotlin Gradle plugin on the root buildscript classpath; when it is only
    // in pluginManagement (this repo) the class is not visible here: assume 1.9.
    val kotlinVersion = runCatching {
        Class.forName("org.jetbrains.kotlin.gradle.plugin.KotlinPluginWrapper")
            .`package`?.implementationVersion?.substringBefore("-release")
    }.getOrNull() ?: "1.9.25"
    extra["kotlinVersion"] = kotlinVersion
    repositories {
        google()
        mavenCentral()
    }
    if (!kotlinVersion.startsWith("1.")) {
        dependencies { classpath("org.jetbrains.kotlin:compose-compiler-gradle-plugin:$kotlinVersion") }
    }
}

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

val kotlin2 = !(extra["kotlinVersion"] as String).startsWith("1.")
if (kotlin2) apply(plugin = "org.jetbrains.kotlin.plugin.compose")

// A plain path: AGP 9 rejects Providers in the source-set API.
val visualAssetsDir = layout.buildDirectory.dir("generated/visualAssets").get().asFile

android {
    namespace = "git.s00d.widgets"
    compileSdk = 34

    defaultConfig {
        minSdk = 21

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")

        val record =
            (project.findProperty("golden.record")?.toString())
                ?: System.getenv("GOLDEN_RECORD")
                ?: "false"
        testInstrumentationRunnerArguments["golden.record"] = record
        val caseFilter =
            (project.findProperty("case")?.toString())
                ?: System.getenv("CASE")
                ?: ""
        if (caseFilter.isNotBlank()) {
            testInstrumentationRunnerArguments["case"] = caseFilter
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        compose = true
    }
    if (!kotlin2) {
        composeOptions {
            kotlinCompilerExtensionVersion = "1.5.15"
        }
    }
    sourceSets {
        getByName("androidTest").assets.srcDir(visualAssetsDir)
    }
    testOptions {
        // Only the androidTest APK uses it: API 34+ images reject install when targetSdk < 23.
        // (defaultConfig.targetSdk no longer exists for libraries in AGP 9.)
        targetSdk = 34
        unitTests.isIncludeAndroidResources = true
        unitTests.isReturnDefaultValues = true
    }
}

// Maintainer visual tests: cases / goldens from this repo (paths relative to the plugin,
// not to the app that includes it).
val syncVisualTestAssets = tasks.register<Sync>("syncVisualTestAssets") {
    from("${projectDir}/../tests/cases") { into("cases") }
    from("${projectDir}/../tests/fixtures") { into("fixtures") }
    from("${projectDir}/../tests/golden/android") { into("golden") }
    into(visualAssetsDir)
}

tasks.matching { it.name.contains("AndroidTest", ignoreCase = true) }.configureEach {
    dependsOn(syncVisualTestAssets)
}

dependencies {
    implementation("androidx.core:core-ktx:1.9.0")
    implementation("androidx.appcompat:appcompat:1.6.0")
    implementation("com.google.android.material:material:1.7.0")
    testImplementation("junit:junit:4.13.2")
    testImplementation("org.robolectric:robolectric:4.13")
    testImplementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.7.3")
    testImplementation("androidx.test:core:1.5.0")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.1")
    androidTestImplementation("androidx.test:runner:1.5.2")
    androidTestImplementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.7.3")
    implementation(project(":tauri-android"))
    implementation("androidx.glance:glance-appwidget:1.1.1")
    implementation("androidx.glance:glance-material3:1.1.1")
    implementation("androidx.datastore:datastore-preferences:1.1.1")
    if (!kotlin2) add("kotlinCompilerPluginClasspath", "androidx.compose.compiler:compiler:1.5.15")
}

tasks.withType<Test>().configureEach {
    val fixtures = file("${rootProject.projectDir}/../tests/fixtures").absolutePath
    val expected = file("${rootProject.projectDir}/../tests/expected/geometry").absolutePath
    val update =
        (project.findProperty("updateSnapshots")?.toString())
            ?: System.getProperty("update.snapshots")
            ?: System.getenv("UPDATE_SNAPSHOTS")
            ?: "false"
    systemProperty("fixtures.root", fixtures)
    systemProperty("expected.geometry", expected)
    systemProperty(
        "expected.pixels",
        file("${rootProject.projectDir}/../out/android-robolectric").absolutePath,
    )
    systemProperty("update.snapshots", update)
    environment("UPDATE_SNAPSHOTS", if (update == "true" || update == "1") "1" else "0")
}

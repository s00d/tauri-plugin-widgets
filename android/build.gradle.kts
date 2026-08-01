plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "git.s00d.widgets"
    compileSdk = 34

    defaultConfig {
        minSdk = 21

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
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
    composeOptions {
        kotlinCompilerExtensionVersion = "1.5.15"
    }
    testOptions {
        unitTests.isIncludeAndroidResources = true
        unitTests.isReturnDefaultValues = true
    }
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
    implementation(project(":tauri-android"))
    implementation("androidx.glance:glance-appwidget:1.1.1")
    implementation("androidx.glance:glance-material3:1.1.1")
    implementation("androidx.datastore:datastore-preferences:1.1.1")
    add("kotlinCompilerPluginClasspath", "androidx.compose.compiler:compiler:1.5.15")
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
        file("${rootProject.projectDir}/../tests/expected/pixels/android").absolutePath,
    )
    systemProperty("update.snapshots", update)
    // Also expose as env for harness helpers.
    environment("UPDATE_SNAPSHOTS", if (update == "true" || update == "1") "1" else "0")
}

package com.h1dr0n.crosssave_cloud

import android.os.Bundle
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    
    // Enable WebView debugging
    if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.KITKAT) {
        android.webkit.WebView.setWebContentsDebuggingEnabled(true)
    }

    checkPermissions()
  }

  private fun checkPermissions() {
    if (android.os.Build.VERSION.SDK_INT < android.os.Build.VERSION_CODES.M) {
        return
    }

    val permissions = arrayOf(
        android.Manifest.permission.READ_EXTERNAL_STORAGE,
        android.Manifest.permission.WRITE_EXTERNAL_STORAGE
    )

    val neededPermissions = permissions.filter {
        androidx.core.content.ContextCompat.checkSelfPermission(this, it) != android.content.pm.PackageManager.PERMISSION_GRANTED
    }

    if (neededPermissions.isNotEmpty()) {
        androidx.core.app.ActivityCompat.requestPermissions(this, neededPermissions.toTypedArray(), 100)
    }
}
}

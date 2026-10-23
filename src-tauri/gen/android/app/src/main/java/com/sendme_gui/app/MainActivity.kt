package com.sendme_gui.app

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.PowerManager
import android.provider.Settings
import androidx.core.content.ContextCompat

class MainActivity : TauriActivity() {
    // Called from Rust (over JNI) when a transfer starts or finishes, so it
    // keeps running at full speed with the app in the background.
    fun startTransferService() {
        // Android 13+ needs this for the "transferring" notification to be visible.
        // The service runs either way.
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
            checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED
        ) {
            requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 1)
        }
        ContextCompat.startForegroundService(this, Intent(this, TransferService::class.java))
        askToIgnoreBatteryOptimizations()
    }

    // Even with a foreground service, Android's deep sleep slows network traffic to a
    // fraction. Being exempt from battery optimization avoids that, so ask once, at the
    // first transfer, with the standard system dialog.
    private fun askToIgnoreBatteryOptimizations() {
        val power = getSystemService(POWER_SERVICE) as PowerManager
        if (power.isIgnoringBatteryOptimizations(packageName)) return
        val prefs = getSharedPreferences("sendme", MODE_PRIVATE)
        if (prefs.getBoolean("asked_battery_exemption", false)) return
        prefs.edit().putBoolean("asked_battery_exemption", true).apply()
        startActivity(
            Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS, Uri.parse("package:$packageName"))
        )
    }

    fun stopTransferService() {
        stopService(Intent(this, TransferService::class.java))
    }
}

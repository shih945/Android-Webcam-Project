package com.soubhagyajit.awa.server

import android.content.Context
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.os.Build
import android.util.Log

/**
 * Publishes the phone as `_awa._tcp.` so the desktop client can discover it
 * on the LAN without a manual IP.  The mDNS name is intentionally based on the
 * device model; Android's NsdManager resolves collisions by suffixing a number.
 */
object AwaDiscovery {
    private const val SERVICE_TYPE = "_awa._tcp."

    private var nsdManager: NsdManager? = null
    private var registrationListener: NsdManager.RegistrationListener? = null

    @Synchronized
    fun register(context: Context, port: Int, protocol: String) {
        unregister()

        try {
            val serviceInfo = NsdServiceInfo().apply {
                serviceName = "AWA-${Build.MODEL}".replace(' ', '-').take(60)
                serviceType = SERVICE_TYPE
                setPort(port)
                setAttribute("proto", protocol)
            }

            val manager = context.applicationContext
                .getSystemService(Context.NSD_SERVICE) as NsdManager
            val listener = object : NsdManager.RegistrationListener {
                override fun onServiceRegistered(info: NsdServiceInfo) {
                    Log.i("AWA", "mDNS registered: ${info.serviceName}")
                }

                override fun onRegistrationFailed(info: NsdServiceInfo, errorCode: Int) {
                    Log.e("AWA", "mDNS registration failed: $errorCode (${info.serviceName})")
                }

                override fun onServiceUnregistered(info: NsdServiceInfo) {
                    Log.i("AWA", "mDNS unregistered: ${info.serviceName}")
                }

                override fun onUnregistrationFailed(info: NsdServiceInfo, errorCode: Int) {
                    Log.e("AWA", "mDNS unregistration failed: $errorCode")
                }
            }

            manager.registerService(serviceInfo, NsdManager.PROTOCOL_DNS_SD, listener)
            nsdManager = manager
            registrationListener = listener
        } catch (e: Exception) {
            Log.e("AWA", "Could not register mDNS service", e)
        }
    }

    @Synchronized
    fun unregister() {
        val manager = nsdManager
        val listener = registrationListener
        nsdManager = null
        registrationListener = null
        if (manager != null && listener != null) {
            try {
                manager.unregisterService(listener)
            } catch (e: Exception) {
                Log.e("AWA", "Could not unregister mDNS service", e)
            }
        }
    }
}

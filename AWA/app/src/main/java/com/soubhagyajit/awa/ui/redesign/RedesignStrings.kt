package com.soubhagyajit.awa.ui.redesign

import android.content.Context
import android.content.res.Configuration
import android.content.res.Resources
import androidx.compose.runtime.staticCompositionLocalOf
import com.soubhagyajit.awa.R
import java.util.Locale

/**
 * All translations live in res/values/strings_redesign.xml (English, the
 * default) and res/values-<language>/strings_redesign.xml.
 *
 * To add a language:
 *  1. add res/values-xx/strings_redesign.xml
 *  2. add an AppLanguage entry to SUPPORTED_LANGUAGES
 * No other UI code needs to change.
 */
data class AppLanguage(
    val tag: String,
    val displayName: String
)

val SUPPORTED_LANGUAGES = listOf(
    AppLanguage("zh-CN", "中文"),
    AppLanguage("en", "English")
)

const val DEFAULT_LANGUAGE = "zh-CN"
private const val PREFS = "awa_redesign_prefs"
private const val KEY_LANGUAGE = "language"

fun loadSavedLanguage(context: Context): String {
    val saved = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        .getString(KEY_LANGUAGE, DEFAULT_LANGUAGE)
    return if (SUPPORTED_LANGUAGES.any { it.tag == saved }) {
        saved ?: DEFAULT_LANGUAGE
    } else {
        DEFAULT_LANGUAGE
    }
}

fun saveLanguage(context: Context, languageTag: String) {
    context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        .edit()
        .putString(KEY_LANGUAGE, languageTag)
        .apply()
}

fun localizedResources(context: Context, languageTag: String): Resources {
    val configuration = Configuration(context.resources.configuration)
    configuration.setLocale(Locale.forLanguageTag(languageTag))
    return context.createConfigurationContext(configuration).resources
}

class RedesignStrings(private val resources: Resources) {
    val live get() = resources.getString(R.string.redesign_live)
    val idle get() = resources.getString(R.string.redesign_idle)
    val preview get() = resources.getString(R.string.redesign_preview)
    val previewOff get() = resources.getString(R.string.redesign_preview_off)
    val readyTitle get() = resources.getString(R.string.redesign_ready_title)
    val readyBody get() = resources.getString(R.string.redesign_ready_body)
    val previewClosedTitle get() = resources.getString(R.string.redesign_preview_closed_title)
    val previewClosedBody get() = resources.getString(R.string.redesign_preview_closed_body)
    val startingTitle get() = resources.getString(R.string.redesign_starting_title)
    val startingBody get() = resources.getString(R.string.redesign_starting_body)
    val startServer get() = resources.getString(R.string.redesign_start_server)
    val stopServer get() = resources.getString(R.string.redesign_stop_server)
    val startShort get() = resources.getString(R.string.redesign_start_short)
    val stopShort get() = resources.getString(R.string.redesign_stop_short)
    val showPreview get() = resources.getString(R.string.redesign_show_preview)
    val hidePreview get() = resources.getString(R.string.redesign_hide_preview)
    val waiting get() = resources.getString(R.string.redesign_waiting)
    val connected get() = resources.getString(R.string.redesign_connected)
    val exp get() = resources.getString(R.string.redesign_exp)
    val zoom get() = resources.getString(R.string.redesign_zoom)
    val focus get() = resources.getString(R.string.redesign_focus)
    val flash get() = resources.getString(R.string.redesign_flash)
    val dim get() = resources.getString(R.string.redesign_dim)
    val auto get() = resources.getString(R.string.redesign_auto)
    val manual get() = resources.getString(R.string.redesign_manual)
    val on get() = resources.getString(R.string.redesign_on)
    val off get() = resources.getString(R.string.redesign_off)
    val settings get() = resources.getString(R.string.redesign_settings)
    val stream get() = resources.getString(R.string.redesign_stream)
    val streamHint get() = resources.getString(R.string.redesign_stream_hint)
    val resolution get() = resources.getString(R.string.redesign_resolution)
    val quality get() = resources.getString(R.string.redesign_quality)
    val language get() = resources.getString(R.string.redesign_language)
    val help get() = resources.getString(R.string.redesign_help)
    val classicUi get() = resources.getString(R.string.redesign_classic_ui)
    val classicHint get() = resources.getString(R.string.redesign_classic_hint)
    val close get() = resources.getString(R.string.redesign_close)
    val exposureTitle get() = resources.getString(R.string.redesign_exposure_title)
    val exposureHint get() = resources.getString(R.string.redesign_exposure_hint)
    val zoomTitle get() = resources.getString(R.string.redesign_zoom_title)
    val zoomHint get() = resources.getString(R.string.redesign_zoom_hint)
    val focusTitle get() = resources.getString(R.string.redesign_focus_title)
    val focusAutoHint get() = resources.getString(R.string.redesign_focus_auto_hint)
    val focusManualHint get() = resources.getString(R.string.redesign_focus_manual_hint)
    val helpTitle get() = resources.getString(R.string.redesign_help_title)
    val helpLead get() = resources.getString(R.string.redesign_help_lead)
    val helpWifi get() = resources.getString(R.string.redesign_help_wifi)
    val helpUsb get() = resources.getString(R.string.redesign_help_usb)
    val copyAddress get() = resources.getString(R.string.redesign_copy_address)
    val copied get() = resources.getString(R.string.redesign_copied)
    val port get() = resources.getString(R.string.redesign_port)
    val permissionsRequired get() = resources.getString(R.string.redesign_permissions_required)
    val waitingForClient get() = resources.getString(R.string.redesign_waiting_for_client)
    val noClient get() = resources.getString(R.string.redesign_no_client)
    val front get() = resources.getString(R.string.redesign_front)
    val back get() = resources.getString(R.string.redesign_back)
}

val LocalRedesignStrings = staticCompositionLocalOf<RedesignStrings> {
    error("RedesignStrings not provided")
}

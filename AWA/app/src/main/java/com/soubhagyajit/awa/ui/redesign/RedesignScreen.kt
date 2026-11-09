package com.soubhagyajit.awa.ui.redesign

import android.content.Intent
import android.content.res.Configuration
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Cameraswitch
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.FlashOff
import androidx.compose.material.icons.filled.FlashOn
import androidx.compose.material.icons.filled.Link
import androidx.compose.material.icons.filled.PowerSettingsNew
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.soubhagyajit.awa.MainActivity
import com.soubhagyajit.awa.checkPermissions
import com.soubhagyajit.awa.getLocalIpAddress
import com.soubhagyajit.awa.requiredPermissions
import com.soubhagyajit.awa.ui.components.Preview
import com.soubhagyajit.awa.viewModel.CameraViewModel

private object RC {
    val bg = Color(0xFF0D1411)
    val panel = Color(0xFF16211B)
    val panel2 = Color(0xFF1D2B24)
    val line = Color(0xFF283A30)
    val paper = Color(0xFFF1EEE6)
    val paper2 = Color(0xFFE7E2D6)
    val ink = Color(0xFF19211C)
    val muted = Color(0xFF68726B)
    val soft = Color(0xFFB5C1B8)
    val faint = Color(0xFF839188)
    val mint = Color(0xFFB9F0D5)
    val mintDeep = Color(0xFF7EE0B8)
    val mintInk = Color(0xFF0D2419)
    val amber = Color(0xFFF1B35A)
    val red = Color(0xFFFF8E7F)
}

private enum class SheetKind { Settings, Exposure, Zoom, Focus, Help }

@Composable
fun RedesignApp(camView: CameraViewModel = viewModel()) {
    val context = LocalContext.current
    var language by rememberSaveable { mutableStateOf(loadSavedLanguage(context)) }
    val strings = remember(language, context) {
        RedesignStrings(localizedResources(context, language))
    }

    CompositionLocalProvider(LocalRedesignStrings provides strings) {
        val hasPermission by checkPermissions(*requiredPermissions())
        if (hasPermission) {
            RedesignContent(
                camView = camView,
                language = language,
                onLanguageChange = {
                    language = it
                    saveLanguage(context, it)
                }
            )
        } else {
            Box(
                modifier = Modifier.fillMaxSize().background(RC.bg).windowInsetsPadding(WindowInsets.safeDrawing),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = strings.permissionsRequired,
                    color = RC.soft,
                    fontSize = 14.sp,
                    modifier = Modifier.padding(32.dp)
                )
            }
        }
    }
}

@Composable
private fun RedesignContent(
    camView: CameraViewModel,
    language: String,
    onLanguageChange: (String) -> Unit
) {
    val context = LocalContext.current
    val isLandscape = LocalConfiguration.current.orientation == Configuration.ORIENTATION_LANDSCAPE
    val isServerRunning by camView.isServerRunning
    val isPreviewActive by camView.isPreviewActive
    val settings by camView.settings
    val streamMode by camView.streamMode

    val copy = LocalRedesignStrings.current
    var previewOn by rememberSaveable { mutableStateOf(true) }
    var sheet by remember { mutableStateOf<SheetKind?>(null) }
    var dimmed by remember { mutableStateOf(false) }
    var copied by remember { mutableStateOf(false) }
    var ip by remember { mutableStateOf(getLocalIpAddress()) }
    val clipboard = LocalClipboardManager.current

    LaunchedEffect(Unit) {
        camView.initialize(context)
        ip = getLocalIpAddress()
    }

    val address = ip ?: "—"
    val url = "http://$address:8080/video"

    fun togglePreview() {
        previewOn = !previewOn
        if (previewOn) camView.startLocalPreview() else camView.stopLocalPreview()
    }

    fun toggleServer() {
        if (isServerRunning) {
            camView.toggleServer()
            previewOn = false
            camView.stopLocalPreview()
        } else {
            camView.toggleServer()
            previewOn = true
            camView.startLocalPreview()
        }
    }

    fun copyAddress() {
        clipboard.setText(AnnotatedString(url))
        copied = true
    }

    LaunchedEffect(copied) {
        if (copied) {
            kotlinx.coroutines.delay(1500)
            copied = false
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(RC.bg)
            .windowInsetsPadding(WindowInsets.safeDrawing)
    ) {
        if (isLandscape) {
            LandscapeLayout(
                camView = camView,
                copy = copy,
                previewOn = previewOn,
                isServerRunning = isServerRunning,
                isPreviewActive = isPreviewActive,
                settings = settings,
                streamMode = streamMode,
                address = address,
                dimmed = dimmed,
                onPreviewToggle = ::togglePreview,
                onPower = ::toggleServer,
                onQuick = { sheet = it },
                onFlash = { camView.toggleFlash() },
                onDim = { dimmed = !dimmed },
                onCamera = { camView.switchCamera() },
                onSettings = { sheet = SheetKind.Settings },
                onCopy = ::copyAddress
            )
        } else {
            PortraitLayout(
                camView = camView,
                copy = copy,
                previewOn = previewOn,
                isServerRunning = isServerRunning,
                isPreviewActive = isPreviewActive,
                settings = settings,
                streamMode = streamMode,
                address = address,
                dimmed = dimmed,
                onPreviewToggle = ::togglePreview,
                onPower = ::toggleServer,
                onQuick = { sheet = it },
                onFlash = { camView.toggleFlash() },
                onDim = { dimmed = !dimmed },
                onCamera = { camView.switchCamera() },
                onSettings = { sheet = SheetKind.Settings },
                onCopy = ::copyAddress
            )
        }

        sheet?.let { kind ->
            SheetHost(
                kind = kind,
                isLandscape = isLandscape,
                lang = language,
                copy = copy,
                camView = camView,
                settings = settings,
                streamMode = streamMode,
                address = address,
                url = url,
                copied = copied,
                onClose = { sheet = null },
                onLanguageChange = onLanguageChange,
                onCopy = ::copyAddress,
                onHelp = { sheet = SheetKind.Help },
                onClassic = {
                    context.startActivity(Intent(context, MainActivity::class.java))
                }
            )
        }
    }
}

/* ------------------------------- portrait ------------------------------- */

@Composable
private fun PortraitLayout(
    camView: CameraViewModel,
    copy: RedesignStrings,
    previewOn: Boolean,
    isServerRunning: Boolean,
    isPreviewActive: Boolean,
    settings: CameraViewModel.CameraSettings,
    streamMode: CameraViewModel.StreamMode,
    address: String,
    dimmed: Boolean,
    onPreviewToggle: () -> Unit,
    onPower: () -> Unit,
    onQuick: (SheetKind) -> Unit,
    onFlash: () -> Unit,
    onDim: () -> Unit,
    onCamera: () -> Unit,
    onSettings: () -> Unit,
    onCopy: () -> Unit
) {
    Column(Modifier.fillMaxSize()) {
        TopBar(
            copy = copy,
            isServerRunning = isServerRunning,
            address = address,
            previewOn = previewOn,
            onPreviewToggle = onPreviewToggle
        )
        Box(
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth()
                .padding(horizontal = 12.dp)
                .clip(RoundedCornerShape(24.dp))
        ) {
            PreviewArea(
                camView = camView,
                copy = copy,
                previewOn = previewOn,
                isServerRunning = isServerRunning,
                isPreviewActive = isPreviewActive,
                settings = settings,
                streamMode = streamMode,
                address = address,
                dimmed = dimmed,
                onPower = onPower,
                onPreviewToggle = onPreviewToggle,
                onCopy = onCopy
            )
        }
        QuickBar(
            copy = copy,
            settings = settings,
            dimmed = dimmed,
            onQuick = onQuick,
            onFlash = onFlash,
            onDim = onDim
        )
        Dock(
            copy = copy,
            isServerRunning = isServerRunning,
            onSettings = onSettings,
            onPower = onPower,
            onCamera = onCamera
        )
    }
}

/* ------------------------------- landscape ------------------------------- */

@Composable
private fun LandscapeLayout(
    camView: CameraViewModel,
    copy: RedesignStrings,
    previewOn: Boolean,
    isServerRunning: Boolean,
    isPreviewActive: Boolean,
    settings: CameraViewModel.CameraSettings,
    streamMode: CameraViewModel.StreamMode,
    address: String,
    dimmed: Boolean,
    onPreviewToggle: () -> Unit,
    onPower: () -> Unit,
    onQuick: (SheetKind) -> Unit,
    onFlash: () -> Unit,
    onDim: () -> Unit,
    onCamera: () -> Unit,
    onSettings: () -> Unit,
    onCopy: () -> Unit
) {
    Row(Modifier.fillMaxSize()) {
        Box(
            modifier = Modifier
                .weight(1f)
                .fillMaxHeight()
                .padding(start = 8.dp, top = 8.dp, bottom = 8.dp)
                .clip(RoundedCornerShape(20.dp))
        ) {
            PreviewArea(
                camView = camView,
                copy = copy,
                previewOn = previewOn,
                isServerRunning = isServerRunning,
                isPreviewActive = isPreviewActive,
                settings = settings,
                streamMode = streamMode,
                address = address,
                dimmed = dimmed,
                landscape = true,
                onPower = onPower,
                onPreviewToggle = onPreviewToggle,
                onCopy = onCopy
            )
        }
        LandscapeConsole(
            copy = copy,
            isServerRunning = isServerRunning,
            settings = settings,
            address = address,
            dimmed = dimmed,
            previewOn = previewOn,
            onPreviewToggle = onPreviewToggle,
            onSettings = onSettings,
            onPower = onPower,
            onCamera = onCamera,
            onQuick = onQuick,
            onFlash = onFlash,
            onDim = onDim
        )
    }
}

@Composable
private fun LandscapeConsole(
    copy: RedesignStrings,
    isServerRunning: Boolean,
    settings: CameraViewModel.CameraSettings,
    address: String,
    dimmed: Boolean,
    previewOn: Boolean,
    onPreviewToggle: () -> Unit,
    onSettings: () -> Unit,
    onPower: () -> Unit,
    onCamera: () -> Unit,
    onQuick: (SheetKind) -> Unit,
    onFlash: () -> Unit,
    onDim: () -> Unit
) {
    Column(
        modifier = Modifier
            .width(300.dp)
            .fillMaxHeight()
            .padding(10.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            StatusLabel(copy = copy, isServerRunning = isServerRunning)
            Text(
                text = "$address:8080",
                color = RC.soft,
                fontSize = 9.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            IconButton(onClick = onPreviewToggle, modifier = Modifier.size(32.dp)) {
                Text(
                    text = if (previewOn) "◉" else "○",
                    color = if (previewOn) RC.mint else RC.soft,
                    fontSize = 16.sp
                )
            }
        }
        Spacer(Modifier.height(4.dp))
        ConsoleControl(copy.exp, formatExposure(settings.exposureIndex), onClick = { onQuick(SheetKind.Exposure) })
        ConsoleControl(copy.zoom, "${"%.1f".format(settings.zoom)}×", onClick = { onQuick(SheetKind.Zoom) })
        ConsoleControl(
            copy.focus,
            if (settings.focusMode == CameraViewModel.FocusMode.AUTO) copy.auto else copy.manual,
            onClick = { onQuick(SheetKind.Focus) }
        )
        ConsoleControl(
            copy.flash,
            if (settings.isFlashEnabled) copy.on else copy.off,
            active = settings.isFlashEnabled,
            onClick = onFlash
        )
        ConsoleControl(
            copy.dim,
            if (dimmed) copy.on else copy.off,
            active = dimmed,
            onClick = onDim
        )
        Spacer(Modifier.weight(1f))
        Row(
            modifier = Modifier.fillMaxWidth().height(42.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            IconButton(onClick = onSettings) {
                Icon(Icons.Default.Settings, contentDescription = copy.settings, tint = RC.soft)
            }
            PowerPill(copy, isServerRunning, onPower)
            IconButton(onClick = onCamera) {
                Icon(Icons.Default.Cameraswitch, contentDescription = "Camera", tint = RC.soft)
            }
        }
    }
}

@Composable
private fun ConsoleControl(
    label: String,
    value: String,
    active: Boolean = false,
    onClick: () -> Unit
) {
    Surface(
        modifier = Modifier.fillMaxWidth().height(40.dp).clip(RoundedCornerShape(10.dp)).clickable(onClick = onClick),
        color = if (active) RC.mint else RC.panel2
    ) {
        Row(
            modifier = Modifier.fillMaxSize().padding(horizontal = 10.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = label,
                color = if (active) RC.mintInk else RC.faint,
                fontSize = 9.sp,
                fontWeight = FontWeight.SemiBold
            )
            Spacer(Modifier.weight(1f))
            Text(
                text = value,
                color = if (active) RC.mintInk else Color.White,
                fontSize = 11.sp,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1
            )
        }
    }
}

/* ------------------------------- shared UI ------------------------------- */

@Composable
private fun TopBar(
    copy: RedesignStrings,
    isServerRunning: Boolean,
    address: String,
    previewOn: Boolean,
    onPreviewToggle: () -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth().height(52.dp).padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        StatusLabel(copy, isServerRunning)
        Spacer(Modifier.weight(1f))
        Text(
            text = if (isServerRunning) "$address:8080" else "—",
            color = RC.soft,
            fontSize = 10.sp,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis
        )
        IconButton(onClick = onPreviewToggle) {
            Text(
                text = if (previewOn) "◉" else "○",
                color = if (previewOn) RC.mint else RC.soft,
                fontSize = 18.sp
            )
        }
    }
}

@Composable
private fun StatusLabel(copy: RedesignStrings, isServerRunning: Boolean) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Box(
            Modifier
                .size(7.dp)
                .clip(CircleShape)
                .background(if (isServerRunning) RC.mintDeep else RC.faint)
        )
        Spacer(Modifier.width(7.dp))
        Text(
            text = if (isServerRunning) copy.live else copy.idle,
            color = Color.White,
            fontSize = 10.sp,
            fontWeight = FontWeight.SemiBold
        )
    }
}

@Composable
private fun PreviewArea(
    camView: CameraViewModel,
    copy: RedesignStrings,
    previewOn: Boolean,
    isServerRunning: Boolean,
    isPreviewActive: Boolean,
    settings: CameraViewModel.CameraSettings,
    streamMode: CameraViewModel.StreamMode,
    address: String,
    dimmed: Boolean,
    landscape: Boolean = false,
    onPower: () -> Unit,
    onPreviewToggle: () -> Unit,
    onCopy: () -> Unit
) {
    var surfaceReady by remember { mutableStateOf(false) }
    var previewStreaming by remember { mutableStateOf(false) }
    val cameraLive = previewOn && isServerRunning

    LaunchedEffect(cameraLive) {
        previewStreaming = false
        if (!cameraLive) {
            surfaceReady = false
            camView.stopLocalPreview()
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(RC.panel2)
            .pointerInput(previewOn, surfaceReady) {
                detectTapGestures { offset ->
                    if (previewOn) {
                        camView.notifyScreenTapped()
                        if (surfaceReady) {
                            camView.tapToFocus(
                                x = offset.x,
                                y = offset.y,
                                width = size.width.toFloat(),
                                height = size.height.toFloat()
                            )
                        }
                    }
                }
            }
    ) {
        if (!cameraLive) {
            IdleCard(
                copy = copy,
                isServerRunning = isServerRunning,
                onPower = onPower,
                onPreviewToggle = onPreviewToggle
            )
        } else {
            Preview(
                camView,
                Modifier.fillMaxSize(),
                onPreviewReady = {
                    surfaceReady = true
                    camView.startLocalPreview()
                },
                onPreviewStreaming = {
                    previewStreaming = true
                }
            )
            if (!previewStreaming) {
                IdleCard(
                    copy = copy,
                    isServerRunning = isServerRunning,
                    starting = true,
                    onPower = onPower,
                    onPreviewToggle = onPreviewToggle
                )
            } else {
                PreviewOverlay(
                    copy = copy,
                    streamMode = streamMode,
                    settings = settings,
                    address = address,
                    connected = isPreviewActive,
                    landscape = landscape,
                    onCopy = onCopy
                )
            }
        }

        if (dimmed) {
            Box(
                Modifier
                    .fillMaxSize()
                    .background(Color.Black.copy(alpha = 0.55f))
                    .clickable { },
                contentAlignment = Alignment.Center
            ) {
                Surface(color = RC.paper, shape = RoundedCornerShape(50)) {
                    Text(
                        copy.dim,
                        color = RC.ink,
                        fontSize = 10.sp,
                        fontWeight = FontWeight.SemiBold,
                        modifier = Modifier.padding(horizontal = 12.dp, vertical = 7.dp)
                    )
                }
            }
        }
    }
}

@Composable
private fun PreviewOverlay(
    copy: RedesignStrings,
    streamMode: CameraViewModel.StreamMode,
    settings: CameraViewModel.CameraSettings,
    address: String,
    connected: Boolean,
    landscape: Boolean,
    onCopy: () -> Unit
) {
    Box(Modifier.fillMaxSize()) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = 10.dp, start = if (landscape) 34.dp else 12.dp, end = 12.dp),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            OverlayChip(copy.preview)
            OverlayChip("${streamMode.label} · ${settings.resolution.key}")
        }
        Row(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .padding(start = if (landscape) 34.dp else 10.dp, end = 10.dp, bottom = 10.dp)
                .background(Color(0xCC0A100D), RoundedCornerShape(10.dp))
                .padding(horizontal = 10.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                Modifier
                    .size(6.dp)
                    .clip(CircleShape)
                    .background(if (connected) RC.mintDeep else RC.faint)
            )
            Spacer(Modifier.width(7.dp))
            Column(Modifier.weight(1f)) {
                Text(
                    text = if (connected) copy.connected else copy.waiting,
                    color = Color.White,
                    fontSize = 10.sp,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Text(
                    text = "$address:8080",
                    color = RC.soft,
                    fontSize = 9.sp,
                    maxLines = 1
                )
            }
            IconButton(onClick = onCopy, modifier = Modifier.size(32.dp)) {
                Icon(Icons.Default.Link, contentDescription = copy.copyAddress, tint = RC.soft, modifier = Modifier.size(15.dp))
            }
        }
    }
}

@Composable
private fun OverlayChip(text: String) {
    Surface(color = Color(0xCC0A100D), shape = RoundedCornerShape(50)) {
        Text(
            text = text,
            color = Color.White,
            fontSize = 9.sp,
            fontWeight = FontWeight.SemiBold,
            modifier = Modifier.padding(horizontal = 9.dp, vertical = 6.dp)
        )
    }
}

@Composable
private fun IdleCard(
    copy: RedesignStrings,
    isServerRunning: Boolean,
    starting: Boolean = false,
    onPower: () -> Unit,
    onPreviewToggle: () -> Unit
) {
    Surface(
        modifier = Modifier.fillMaxSize().clip(RoundedCornerShape(24.dp)),
        color = RC.paper
    ) {
        Box(Modifier.fillMaxSize()) {
            Column(
                modifier = Modifier.fillMaxSize().padding(22.dp),
                verticalArrangement = Arrangement.SpaceBetween
            ) {
                Text(
                    text = copy.previewOff,
                    color = RC.muted,
                    fontSize = 10.sp
                )
                Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    Surface(color = RC.paper2, shape = RoundedCornerShape(16.dp)) {
                        Text(
                            text = "◎",
                            color = RC.ink,
                            fontSize = 24.sp,
                            modifier = Modifier.padding(10.dp)
                        )
                    }
                    Text(
                        text = when {
                            starting -> copy.startingTitle
                            isServerRunning -> copy.previewClosedTitle
                            else -> copy.readyTitle
                        },
                        color = RC.ink,
                        fontSize = 28.sp,
                        fontWeight = FontWeight.SemiBold,
                        letterSpacing = (-0.8).sp
                    )
                    Text(
                        text = when {
                            starting -> copy.startingBody
                            isServerRunning -> copy.previewClosedBody
                            else -> copy.readyBody
                        },
                        color = RC.muted,
                        fontSize = 12.sp,
                        lineHeight = 18.sp,
                        modifier = Modifier.widthIn(max = 280.dp)
                    )
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Button(
                            onClick = when {
                                starting -> onPreviewToggle
                                isServerRunning -> onPreviewToggle
                                else -> onPower
                            },
                            colors = ButtonDefaults.buttonColors(containerColor = RC.ink, contentColor = RC.paper),
                            shape = RoundedCornerShape(50),
                            contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 14.dp),
                            modifier = Modifier.height(40.dp)
                        ) {
                            Text(
                                text = when {
                                    starting -> copy.hidePreview
                                    isServerRunning -> copy.showPreview
                                    else -> copy.startServer
                                },
                                fontSize = 11.sp,
                                fontWeight = FontWeight.SemiBold
                            )
                        }
                        if (isServerRunning) {
                            TextButton(onClick = onPower) {
                                Text(copy.stopServer, color = RC.ink, fontSize = 11.sp)
                            }
                        }
                    }
                }
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    IdleChip("USB")
                    IdleChip("Wi-Fi")
                    IdleChip(copy.port)
                }
            }
            CornerBrackets(color = RC.paper2)
        }
    }
}

@Composable
private fun IdleChip(text: String) {
    Surface(color = RC.paper2, shape = RoundedCornerShape(50)) {
        Text(
            text = text,
            color = RC.muted,
            fontSize = 9.sp,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 5.dp)
        )
    }
}

@Composable
private fun CornerBrackets(color: Color) {
    Canvas(Modifier.fillMaxSize().padding(10.dp)) {
        val stroke = 1.5.dp.toPx()
        val len = 18.dp.toPx()
        drawLine(color, Offset(0f, 0f), Offset(len, 0f), stroke)
        drawLine(color, Offset(0f, 0f), Offset(0f, len), stroke)
        drawLine(color, Offset(size.width, 0f), Offset(size.width - len, 0f), stroke)
        drawLine(color, Offset(size.width, 0f), Offset(size.width, len), stroke)
        drawLine(color, Offset(0f, size.height), Offset(len, size.height), stroke)
        drawLine(color, Offset(0f, size.height), Offset(0f, size.height - len), stroke)
        drawLine(color, Offset(size.width, size.height), Offset(size.width - len, size.height), stroke)
        drawLine(color, Offset(size.width, size.height), Offset(size.width, size.height - len), stroke)
    }
}

@Composable
private fun DrawnViewfinder() {
    Canvas(Modifier.fillMaxSize()) {
        val w = size.width
        val h = size.height
        drawRect(Color(0xFFC9C5B6), size = size)
        drawRect(Color(0xFFBCB7A8), size = Size(w * 0.36f, h))
        drawRect(Color(0xFFECE9DD), topLeft = Offset(w * 0.48f, h * 0.1f), size = Size(w * 0.47f, h * 0.42f))
        drawLine(Color(0xFFB9B3A3), Offset(w * 0.715f, h * 0.1f), Offset(w * 0.715f, h * 0.52f), 1.5.dp.toPx())
        drawRect(Color(0xFF7A614B), topLeft = Offset(0f, h * 0.64f), size = Size(w, h * 0.36f))
        drawRect(Color(0xFF96765B), topLeft = Offset(0f, h * 0.64f), size = Size(w, 12.dp.toPx()))
        drawCircle(Color(0xFF2F5947), radius = h * 0.075f, center = Offset(w * 0.15f, h * 0.64f))
        drawCircle(Color(0xFF447A5D), radius = h * 0.055f, center = Offset(w * 0.21f, h * 0.61f))
        drawRoundRect(
            color = Color(0xFFB66F4D),
            topLeft = Offset(w * 0.1f, h * 0.65f),
            size = Size(w * 0.1f, h * 0.1f),
            cornerRadius = CornerRadius(4.dp.toPx())
        )
        drawRoundRect(
            color = Color(0xFF252D28),
            topLeft = Offset(w * 0.62f, h * 0.58f),
            size = Size(w * 0.22f, h * 0.14f),
            cornerRadius = CornerRadius(8.dp.toPx())
        )
        drawCircle(
            color = Color(0xFF1A211D),
            radius = h * 0.055f,
            center = Offset(w * 0.73f, h * 0.65f),
            style = Stroke(width = 2.dp.toPx())
        )
        val focus = Path().apply {
            moveTo(w * 0.47f, h * 0.43f)
            lineTo(w * 0.42f, h * 0.43f)
            lineTo(w * 0.42f, h * 0.38f)
        }
        drawPath(focus, Color.White.copy(alpha = 0.35f), style = Stroke(1.dp.toPx()))
    }
}

@Composable
private fun QuickBar(
    copy: RedesignStrings,
    settings: CameraViewModel.CameraSettings,
    dimmed: Boolean,
    onQuick: (SheetKind) -> Unit,
    onFlash: () -> Unit,
    onDim: () -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(start = 12.dp, end = 12.dp, top = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp)
    ) {
        QuickControl(
            label = copy.exp,
            value = formatExposure(settings.exposureIndex),
            modifier = Modifier.weight(1f),
            onClick = { onQuick(SheetKind.Exposure) }
        )
        QuickControl(
            label = copy.zoom,
            value = "${"%.1f".format(settings.zoom)}×",
            modifier = Modifier.weight(1f),
            onClick = { onQuick(SheetKind.Zoom) }
        )
        QuickControl(
            label = copy.focus,
            value = if (settings.focusMode == CameraViewModel.FocusMode.AUTO) copy.auto else copy.manual,
            modifier = Modifier.weight(1f),
            onClick = { onQuick(SheetKind.Focus) }
        )
        QuickControl(
            label = copy.flash,
            value = if (settings.isFlashEnabled) copy.on else copy.off,
            active = settings.isFlashEnabled,
            modifier = Modifier.weight(1f),
            onClick = onFlash
        )
        QuickControl(
            label = copy.dim,
            value = if (dimmed) copy.on else copy.off,
            active = dimmed,
            modifier = Modifier.weight(1f),
            onClick = onDim
        )
    }
}

@Composable
private fun QuickControl(
    label: String,
    value: String,
    modifier: Modifier = Modifier,
    active: Boolean = false,
    onClick: () -> Unit
) {
    Surface(
        modifier = modifier.heightIn(min = 52.dp).clip(RoundedCornerShape(14.dp)).clickable(onClick = onClick),
        color = if (active) RC.mint else RC.panel
    ) {
        Column(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 8.dp, vertical = 7.dp),
            verticalArrangement = Arrangement.Center
        ) {
            Text(
                text = label,
                color = if (active) RC.mintInk else RC.faint,
                fontSize = 8.sp,
                fontWeight = FontWeight.SemiBold,
                lineHeight = 10.sp,
                maxLines = 1
            )
            Spacer(Modifier.height(3.dp))
            Text(
                text = value,
                color = if (active) RC.mintInk else Color.White,
                fontSize = 11.sp,
                fontWeight = FontWeight.SemiBold,
                lineHeight = 13.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
    }
}

@Composable
private fun Dock(
    copy: RedesignStrings,
    isServerRunning: Boolean,
    onSettings: () -> Unit,
    onPower: () -> Unit,
    onCamera: () -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth().height(64.dp).padding(horizontal = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        IconButton(onClick = onSettings) {
            Icon(Icons.Default.Settings, contentDescription = copy.settings, tint = RC.soft)
        }
        PowerPill(copy, isServerRunning, onPower)
        IconButton(onClick = onCamera) {
            Icon(Icons.Default.Cameraswitch, contentDescription = "Camera", tint = RC.soft)
        }
    }
}

@Composable
private fun PowerPill(copy: RedesignStrings, isServerRunning: Boolean, onClick: () -> Unit) {
    Surface(
        modifier = Modifier.width(96.dp).height(40.dp).clip(RoundedCornerShape(50)).clickable(onClick = onClick),
        color = if (isServerRunning) RC.mint else RC.paper
    ) {
        Row(
            modifier = Modifier.fillMaxSize(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.Center
        ) {
            Icon(
                Icons.Default.PowerSettingsNew,
                contentDescription = null,
                tint = RC.ink,
                modifier = Modifier.size(17.dp)
            )
            Spacer(Modifier.width(6.dp))
            Text(
                text = if (isServerRunning) copy.stopShort else copy.startShort,
                color = RC.ink,
                fontSize = 11.sp,
                fontWeight = FontWeight.SemiBold
            )
        }
    }
}

/* -------------------------------- sheets -------------------------------- */

@Composable
private fun SheetHost(
    kind: SheetKind,
    isLandscape: Boolean,
    lang: String,
    copy: RedesignStrings,
    camView: CameraViewModel,
    settings: CameraViewModel.CameraSettings,
    streamMode: CameraViewModel.StreamMode,
    address: String,
    url: String,
    copied: Boolean,
    onClose: () -> Unit,
    onLanguageChange: (String) -> Unit,
    onCopy: () -> Unit,
    onHelp: () -> Unit,
    onClassic: () -> Unit
) {
    Box(Modifier.fillMaxSize()) {
        Box(
            Modifier
                .fillMaxSize()
                .background(Color.Black.copy(alpha = 0.58f))
                .clickable(onClick = onClose)
        )
        Surface(
            modifier = if (isLandscape) {
                Modifier.align(Alignment.CenterEnd).fillMaxHeight().width(340.dp)
            } else {
                Modifier.align(Alignment.BottomCenter).fillMaxWidth().heightIn(max = 620.dp).fillMaxHeight(0.76f)
            },
            color = RC.panel,
            shape = if (isLandscape) RoundedCornerShape(topStart = 24.dp, bottomStart = 24.dp)
            else RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp)
        ) {
            Column(Modifier.fillMaxSize().padding(16.dp)) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = when (kind) {
                            SheetKind.Settings -> copy.settings
                            SheetKind.Exposure -> copy.exposureTitle
                            SheetKind.Zoom -> copy.zoomTitle
                            SheetKind.Focus -> copy.focusTitle
                            SheetKind.Help -> copy.helpTitle
                        },
                        color = Color.White,
                        fontSize = 18.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                    Spacer(Modifier.weight(1f))
                    IconButton(onClick = onClose) {
                        Icon(Icons.Default.Close, contentDescription = copy.close, tint = RC.soft)
                    }
                }
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .weight(1f)
                        .verticalScroll(rememberScrollState())
                ) {
                    when (kind) {
                        SheetKind.Settings -> SettingsContent(
                            copy = copy,
                            lang = lang,
                            camView = camView,
                            settings = settings,
                            streamMode = streamMode,
                            onLanguageChange = onLanguageChange,
                            onHelp = onHelp,
                            onClassic = onClassic
                        )
                        SheetKind.Exposure -> ExposureContent(copy, camView, settings)
                        SheetKind.Zoom -> ZoomContent(copy, camView, settings)
                        SheetKind.Focus -> FocusContent(copy, camView, settings)
                        SheetKind.Help -> HelpContent(copy, url, copied, onCopy)
                    }
                }
            }
        }
    }
}

@Composable
private fun SettingsContent(
    copy: RedesignStrings,
    lang: String,
    camView: CameraViewModel,
    settings: CameraViewModel.CameraSettings,
    streamMode: CameraViewModel.StreamMode,
    onLanguageChange: (String) -> Unit,
    onHelp: () -> Unit,
    onClassic: () -> Unit
) {
    var resolutionExpanded by remember { mutableStateOf(false) }
    var streamExpanded by remember { mutableStateOf(false) }
    SettingLabel(copy.stream, copy.streamHint)
    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        SegmentButton(
            text = "MJPEG",
            selected = streamMode == CameraViewModel.StreamMode.MJPEG,
            onClick = { camView.setStreamMode(CameraViewModel.StreamMode.MJPEG) }
        )
        SegmentButton(
            text = "RTSP",
            selected = streamMode == CameraViewModel.StreamMode.H264_RTSP,
            onClick = { camView.setStreamMode(CameraViewModel.StreamMode.H264_RTSP) }
        )
    }
    Spacer(Modifier.height(14.dp))
    SettingLabel(copy.resolution, settings.resolution.label)
    Box {
        SettingRow(text = settings.resolution.key, onClick = { resolutionExpanded = true })
        DropdownMenu(expanded = resolutionExpanded, onDismissRequest = { resolutionExpanded = false }) {
            settings.supportedResolutions.forEach { resolution ->
                DropdownMenuItem(
                    text = { Text(resolution.label) },
                    onClick = {
                        camView.setResolution(resolution)
                        resolutionExpanded = false
                    }
                )
            }
        }
    }
    if (streamMode == CameraViewModel.StreamMode.MJPEG) {
        Spacer(Modifier.height(10.dp))
        SettingLabel(copy.quality, "${settings.jpegQuality}")
        Slider(
            value = settings.jpegQuality.toFloat(),
            onValueChange = { camView.setQuality(it.toInt()) },
            valueRange = 10f..100f,
            colors = SliderDefaults.colors(
                thumbColor = RC.mintDeep,
                activeTrackColor = RC.mintDeep,
                inactiveTrackColor = RC.line
            )
        )
    }
    Spacer(Modifier.height(12.dp))
    SettingLabel(copy.language, "")
    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        SUPPORTED_LANGUAGES.forEach { languageOption ->
            SegmentButton(
                text = languageOption.displayName,
                selected = lang == languageOption.tag,
                onClick = { onLanguageChange(languageOption.tag) }
            )
        }
    }
    Spacer(Modifier.height(16.dp))
    SettingRow(text = copy.help, onClick = onHelp)
    Spacer(Modifier.height(6.dp))
    SettingRow(text = copy.classicUi, hint = copy.classicHint, onClick = onClassic)
}

@Composable
private fun ExposureContent(copy: RedesignStrings, camView: CameraViewModel, settings: CameraViewModel.CameraSettings) {
    val first = settings.exposureRange.first
    val last = settings.exposureRange.last
    val range = if (last > first) first.toFloat()..last.toFloat() else 0f..1f
    Text(
        text = "${formatExposure(settings.exposureIndex)} EV",
        color = Color.White,
        fontSize = 34.sp,
        fontWeight = FontWeight.Medium
    )
    Text(copy.exposureHint, color = RC.faint, fontSize = 10.sp, modifier = Modifier.padding(vertical = 8.dp))
    Slider(
        value = settings.exposureIndex.toFloat().coerceIn(range),
        onValueChange = { camView.setExposure(it.toInt()) },
        valueRange = range,
        colors = SliderDefaults.colors(thumbColor = RC.mintDeep, activeTrackColor = RC.mintDeep, inactiveTrackColor = RC.line)
    )
}

@Composable
private fun ZoomContent(copy: RedesignStrings, camView: CameraViewModel, settings: CameraViewModel.CameraSettings) {
    val range = settings.zoomMin..maxOf(settings.zoomMax, settings.zoomMin + 0.1f)
    Text(
        text = "${"%.1f".format(settings.zoom)}×",
        color = Color.White,
        fontSize = 34.sp,
        fontWeight = FontWeight.Medium
    )
    Text(copy.zoomHint, color = RC.faint, fontSize = 10.sp, modifier = Modifier.padding(vertical = 8.dp))
    Slider(
        value = settings.zoom.coerceIn(range),
        onValueChange = { camView.setZoom(it) },
        valueRange = range,
        colors = SliderDefaults.colors(thumbColor = RC.mintDeep, activeTrackColor = RC.mintDeep, inactiveTrackColor = RC.line)
    )
}

@Composable
private fun FocusContent(copy: RedesignStrings, camView: CameraViewModel, settings: CameraViewModel.CameraSettings) {
    val manual = settings.focusMode == CameraViewModel.FocusMode.MANUAL
    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        SegmentButton(copy.auto, !manual) { if (manual) camView.toggleFocusMode() }
        SegmentButton(copy.manual, manual) { if (!manual) camView.toggleFocusMode() }
    }
    Spacer(Modifier.height(12.dp))
    Text(
        text = if (manual) copy.focusManualHint else copy.focusAutoHint,
        color = RC.faint,
        fontSize = 10.sp
    )
    if (manual) {
        Slider(
            value = settings.focusDistance,
            onValueChange = { camView.setFocusDistance(it) },
            valueRange = 0f..1f,
            colors = SliderDefaults.colors(thumbColor = RC.mintDeep, activeTrackColor = RC.mintDeep, inactiveTrackColor = RC.line)
        )
    }
}

@Composable
private fun HelpContent(copy: RedesignStrings, url: String, copied: Boolean, onCopy: () -> Unit) {
    Text(copy.helpLead, color = RC.soft, fontSize = 11.sp, lineHeight = 16.sp)
    Spacer(Modifier.height(10.dp))
    Surface(color = RC.bg, shape = RoundedCornerShape(10.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(10.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(url, color = RC.mint, fontSize = 10.sp, modifier = Modifier.weight(1f), maxLines = 1, overflow = TextOverflow.Ellipsis)
            TextButton(onClick = onCopy) { Text(if (copied) copy.copied else copy.copyAddress, fontSize = 9.sp) }
        }
    }
    Spacer(Modifier.height(10.dp))
    Text(copy.helpWifi, color = RC.soft, fontSize = 10.sp, modifier = Modifier.padding(vertical = 4.dp))
    Text(copy.helpUsb, color = RC.soft, fontSize = 10.sp, modifier = Modifier.padding(vertical = 4.dp))
}

@Composable
private fun SettingLabel(label: String, hint: String) {
    Text(label, color = Color.White, fontSize = 11.sp, fontWeight = FontWeight.SemiBold)
    if (hint.isNotBlank()) {
        Text(hint, color = RC.faint, fontSize = 10.sp, lineHeight = 14.sp, modifier = Modifier.padding(top = 2.dp))
    }
}

@Composable
private fun SettingRow(text: String, hint: String = "", onClick: () -> Unit) {
    Surface(
        modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp)).clickable(onClick = onClick),
        color = RC.panel2
    ) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 11.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(Modifier.weight(1f)) {
                Text(text, color = Color.White, fontSize = 11.sp, fontWeight = FontWeight.SemiBold)
                if (hint.isNotBlank()) Text(hint, color = RC.faint, fontSize = 9.sp)
            }
            Text("›", color = RC.soft, fontSize = 18.sp)
        }
    }
}

@Composable
private fun SegmentButton(text: String, selected: Boolean, onClick: () -> Unit) {
    Surface(
        modifier = Modifier.height(32.dp).clip(RoundedCornerShape(50)).clickable(onClick = onClick),
        color = if (selected) RC.paper else RC.panel2
    ) {
        Box(
            Modifier
                .fillMaxHeight()
                .padding(horizontal = 12.dp),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = text,
                color = if (selected) RC.ink else RC.soft,
                fontSize = 10.sp,
                fontWeight = FontWeight.SemiBold,
                textAlign = androidx.compose.ui.text.style.TextAlign.Center
            )
        }
    }
}

private fun formatExposure(index: Int): String {
    if (index == 0) return "0.0"
    return if (index > 0) "+$index" else "−${-index}"
}

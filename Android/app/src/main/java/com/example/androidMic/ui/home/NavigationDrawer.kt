package com.example.androidMic.ui.home

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Divider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.example.androidMic.R
import com.example.androidMic.ui.Event
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.home.dialog.DialogIpPort
import com.example.androidMic.ui.home.dialog.DialogMode
import com.example.androidMic.ui.home.dialog.DialogTheme
import com.example.androidMic.ui.home.dialog.DialogUsbPort
import com.example.androidMic.utils.Modes.Companion.MODE_BLUETOOTH
import com.example.androidMic.utils.Modes.Companion.MODE_USB
import com.example.androidMic.utils.Modes.Companion.MODE_WIFI
import com.example.androidMic.utils.States
import com.example.androidMic.utils.Themes.Companion.DARK_THEME
import com.example.androidMic.utils.Themes.Companion.LIGHT_THEME
import com.example.androidMic.utils.Themes.Companion.SYSTEM_THEME

data class MenuItem(
    val id: Int,
    val title: String,
    val subTitle: String,
    val contentDescription: String,
    val icon: Int
)

@Composable
fun DrawerBody(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    DialogIpPort(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogUsbPort(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogMode(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogTheme(mainViewModel = mainViewModel, uiStates = uiStates)


    val items = listOf(
        MenuItem(
            id = R.string.drawerMode,
            title = stringResource(id = R.string.drawerMode),
            subTitle = when (uiStates.mode) {
                MODE_WIFI -> stringResource(id = R.string.mode_wifi)
                MODE_BLUETOOTH -> stringResource(id = R.string.mode_bluetooth)
                MODE_USB -> stringResource(id = R.string.mode_usb)
                else -> "NONE"
            },
            contentDescription = "set mode",
            icon = R.drawable.settings_24px
        ),
        MenuItem(
            id = R.string.drawerIpPort,
            title = stringResource(id = R.string.drawerIpPort),
            subTitle = uiStates.IP + ":" + uiStates.PORT,
            contentDescription = "set ip and port",
            icon = R.drawable.wifi_24px

        ),
        MenuItem(
            id = R.string.drawerUsbPort,
            title = stringResource(id = R.string.drawerUsbPort),
            subTitle = uiStates.usbPort,
            contentDescription = "set usb port",
            icon = R.drawable.usb_24px

        ),
        MenuItem(
            id = R.string.drawerTheme,
            title = stringResource(id = R.string.drawerTheme),
            subTitle = when (uiStates.theme) {
                SYSTEM_THEME -> stringResource(id = R.string.system_theme)
                LIGHT_THEME -> stringResource(id = R.string.light_theme)
                DARK_THEME -> stringResource(id = R.string.dark_theme)
                else -> "NONE"
            },
            contentDescription = "set theme",
            icon = R.drawable.dark_mode_24px
        )
    )

    LazyColumn {
        // setting title
        item {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 64.dp)
                    .padding(start = 25.dp)
            ) {
                Text(
                    text = stringResource(id = R.string.drawerHeader),
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.onBackground
                )
            }
            Divider(color = MaterialTheme.colorScheme.onBackground)
        }
        items(items) { item ->
            var shouldShowItem = true
            when (uiStates.mode) {
                MODE_WIFI -> if (item.id == R.string.drawerUsbPort) shouldShowItem = false
                MODE_USB -> if (item.id == R.string.drawerIpPort) shouldShowItem = false
                MODE_BLUETOOTH -> {
                    if (item.id == R.string.drawerUsbPort) shouldShowItem = false
                    if (item.id == R.string.drawerIpPort) shouldShowItem = false
                }
            }
            if (shouldShowItem) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable {
                            mainViewModel.onEvent(Event.ShowDialog(item.id))
                        }
                        .padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        painter = painterResource(id = item.icon),
                        contentDescription = item.contentDescription,
                        tint = MaterialTheme.colorScheme.onBackground
                    )
                    Spacer(modifier = Modifier.width(16.dp))
                    Column {
                        Text(
                            text = item.title,
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onBackground
                        )

                        Text(
                            text = item.subTitle,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onBackground
                        )
                    }
                }
                Divider(color = MaterialTheme.colorScheme.onBackground)
            }
        }
    }
}
import { useEffect, useState } from "react";
import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import { isEnabled as isAutostartEnabled, enable as enableAutostart, disable as disableAutostart } from "@tauri-apps/plugin-autostart";
import { api } from "../lib/api";

// macOS never re-prompts once denied — from then on the only way back is
// the OS Notifications settings pane.
export type NotificationPermission = "granted" | "denied" | "default";

/**
 * Tray badge / launch-at-login / notification-permission state, fetched
 * once here at the app root instead of inside Settings. Settings only
 * mounts when the user navigates to it, so fetching there means every
 * switch starts from its useState default and visibly flips once the
 * real value loads. Fetching at the root means the real value is almost
 * always in hand well before Settings is ever opened.
 */
export function useAppPreferences() {
  const [showBadge, setShowBadgeState] = useState<boolean | null>(null);
  const [launchAtLogin, setLaunchAtLoginState] = useState<boolean | null>(null);
  const [notifPermission, setNotifPermission] = useState<NotificationPermission>("default");

  useEffect(() => {
    api
      .getShowTrayBadge()
      .then(setShowBadgeState)
      .catch(() => setShowBadgeState(true));
    isAutostartEnabled()
      .then(setLaunchAtLoginState)
      .catch(() => setLaunchAtLoginState(false));
    // Ask for notification permission once, on first-ever launch, so
    // tunnel-death alerts work without a trip to Settings. Only fires when
    // macOS hasn't recorded a decision yet — a prior denial or grant is
    // left alone, since re-prompting past that is either a no-op or just
    // annoying.
    isPermissionGranted().then(async (granted) => {
      if (granted) {
        setNotifPermission("granted");
        return;
      }
      const result = await requestPermission();
      setNotifPermission(result);
    });
  }, []);

  async function toggleBadge(next: boolean) {
    setShowBadgeState(next);
    try {
      await api.setShowTrayBadge(next);
    } catch {
      setShowBadgeState(!next);
    }
  }

  async function toggleLaunchAtLogin(next: boolean) {
    setLaunchAtLoginState(next);
    try {
      await (next ? enableAutostart() : disableAutostart());
    } catch {
      setLaunchAtLoginState(!next);
    }
  }

  async function enableNotifications() {
    const result = await requestPermission();
    setNotifPermission(result);
  }

  return {
    // Default to the steady-state value (badge on, autostart off) while
    // the real value is still in flight, so a Settings open that beats
    // the fetch still renders correctly rather than flashing the opposite.
    showBadge: showBadge ?? true,
    toggleBadge,
    launchAtLogin: launchAtLogin ?? false,
    toggleLaunchAtLogin,
    notifPermission,
    enableNotifications,
  };
}

use std::process::Command;

pub fn show_toast_notification(title: &str, message: &str) {
    // First try native Windows Runtime API
    if show_winrt_toast(title, message).is_ok() {
        return;
    }

    // Fallback: PowerShell Windows Runtime Toast Notification
    let _ = show_powershell_toast(title, message);
}

#[cfg(windows)]
fn show_winrt_toast(title: &str, message: &str) -> windows::core::Result<()> {
    use windows::core::HSTRING;
    use windows::Data::Xml::Dom::XmlDocument;
    use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

    let xml = format!(
        "<toast><visual><binding template=\"ToastGeneric\"><text>{}</text><text>{}</text></binding></visual></toast>",
        quick_xml_escape(title),
        quick_xml_escape(message)
    );

    let doc = XmlDocument::new()?;
    doc.LoadXml(&HSTRING::from(xml))?;

    let toast = ToastNotification::CreateToastNotification(&doc)?;
    
    // Application ID for the toast notification
    let app_id = HSTRING::from("SavingsTracker.App");
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&app_id)?;
    notifier.Show(&toast)?;
    Ok(())
}

#[cfg(not(windows))]
fn show_winrt_toast(_title: &str, _message: &str) -> Result<(), ()> {
    Err(())
}

fn quick_xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn show_powershell_toast(title: &str, message: &str) -> std::io::Result<()> {
    let script = format!(
        "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null; \
        [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null; \
        $template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); \
        $nodes = $template.GetElementsByTagName('text'); \
        $nodes.Item(0).AppendChild($template.CreateTextNode('{}')) | Out-Null; \
        $nodes.Item(1).AppendChild($template.CreateTextNode('{}')) | Out-Null; \
        $toast = [Windows.UI.Notifications.ToastNotification]::new($template); \
        [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Savings Tracker').Show($toast);",
        title.replace('\'', "''"),
        message.replace('\'', "''")
    );

    Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
        .spawn()?;

    Ok(())
}

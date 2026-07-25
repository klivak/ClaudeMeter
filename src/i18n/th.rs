use std::collections::HashMap;

pub fn strings() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("5-hour session", "เซสชัน 5 ชั่วโมง");
    m.insert("Weekly (7-day)", "รายสัปดาห์ (7 วัน)");
    m.insert("Opus (7-day)", "Opus (7 วัน)");
    m.insert("Sonnet (7-day)", "Sonnet (7 วัน)");
    m.insert("OAuth Apps (7-day)", "OAuth Apps (7 วัน)");
    m.insert("resets in", "รีเซ็ตใน");
    m.insert("Plan", "แผน");
    m.insert("Pro", "Pro");
    m.insert("Max", "Max");
    m.insert("Claude Code not detected", "ไม่พบ Claude Code");
    m.insert("credentials_not_found", "ไม่พบข้อมูลรับรอง");
    m.insert("connection_error", "ข้อผิดพลาดในการเชื่อมต่อ");
    m.insert("token_expired", "โทเค็นหมดอายุ");
    m.insert(
        "token_expired_desc",
        "โทเค็น OAuth ของคุณหมดอายุแล้ว เรียกใช้ `claude login` ในเทอร์มินัลเพื่อต่ออายุ",
    );
    m.insert("rate_limited", "ถูกจำกัดอัตราการใช้งาน");
    m.insert("server_error", "ข้อผิดพลาดของเซิร์ฟเวอร์");
    m.insert(
        "server_error_desc",
        "Anthropic API ไม่พร้อมใช้งานชั่วคราว จะลองใหม่โดยอัตโนมัติ",
    );
    m.insert(
        "run_claude_login_desc",
        "ติดตั้ง Claude Code แล้วแต่ยังไม่ได้เข้าสู่ระบบ เรียกใช้ `claude login` ในเทอร์มินัลเพื่อเชื่อมต่อบัญชี",
    );
    m.insert(
        "install_claude_desc",
        "ติดตั้ง Claude Code และเรียกใช้ `claude login` เพื่อเปิดใช้งานการติดตามการใช้งานอัตโนมัติ",
    );
    m.insert("Install Claude Code \u{2192}", "ติดตั้ง Claude Code \u{2192}");
    m.insert(
        "openai_no_api",
        "OpenAI ไม่มี API สำหรับติดตามการใช้งานสมาชิก ChatGPT",
    );
    m.insert("Check your usage manually:", "ตรวจสอบการใช้งานด้วยตนเอง:");
    m.insert(
        "Open ChatGPT Usage \u{2192}",
        "เปิดการใช้งาน ChatGPT \u{2192}",
    );
    m.insert("Refresh Now", "รีเฟรชตอนนี้");
    m.insert("Open Dashboard", "เปิดแดชบอร์ด");
    m.insert("Export History (CSV)", "ส่งออกประวัติ (CSV)");
    m.insert("Export History (JSON)", "ส่งออกประวัติ (JSON)");
    m.insert("Show extra usage", "แสดงการใช้งานเพิ่มเติม");
    m.insert("Show model limits", "\u{e41}\u{e2a}\u{e14}\u{e07}\u{e02}\u{e35}\u{e14}\u{e08}\u{e33}\u{e01}\u{e31}\u{e14}\u{e02}\u{e2d}\u{e07}\u{e42}\u{e21}\u{e40}\u{e14}\u{e25}");
    m.insert("Usage link icons", "ไอคอนลิงก์การใช้งาน");
    m.insert("Open usage", "เปิดการใช้งาน");
    m.insert("Service status", "สถานะบริการ");
    m.insert("CODEX", "CODEX");
    m.insert("Settings", "การตั้งค่า");
    m.insert("Start with Windows", "เริ่มต้นกับ Windows");
    m.insert("About", "เกี่ยวกับ");
    m.insert("Exit", "ออก");
    m.insert("Last updated:", "อัปเดตล่าสุด:");
    m.insert("Refresh", "รีเฟรช");
    m.insert("Status", "สถานะ");
    m.insert("Usage Alert", "แจ้งเตือนการใช้งาน");
    m.insert("Usage Critical", "การใช้งานวิกฤต");
    m.insert(
        "Running in system tray. Click the icon for details.",
        "ทำงานในซิสเต็มเทรย์ คลิกไอคอนเพื่อดูรายละเอียด",
    );
    m.insert("Compact mode", "โหมดกะทัดรัด");
    m.insert("Theme", "ธีม");
    m.insert("Language", "ภาษา");
    m.insert("Notifications", "การแจ้งเตือน");
    m.insert("Dark", "มืด");
    m.insert("Light", "สว่าง");
    m.insert("Auto", "อัตโนมัติ");
    m.insert("Show ChatGPT section", "แสดงส่วน ChatGPT");
    m.insert("Enabled", "เปิดใช้งาน");
    m.insert("Sound", "เสียง");
    m.insert("Thresholds", "เกณฑ์");
    m.insert("Polling interval", "ช่วงเวลาอัปเดต");
    m.insert("seconds", "วินาที");
    m.insert("Startup", "เริ่มต้น");
    m.insert("General", "ทั่วไป");
    m.insert("Back", "\u{2190} กลับ");
    m.insert("Open Claude.ai \u{2192}", "เปิด Claude.ai \u{2192}");
    m.insert("ClaudeMeter", "ClaudeMeter");
    m.insert("CLAUDE", "CLAUDE");
    m.insert("CHATGPT / CODEX", "CHATGPT / CODEX");
    m.insert("Usage History", "ประวัติการใช้งาน");
    m.insert("Usage History (24h)", "ประวัติการใช้งาน (24 ชม.)");
    m.insert("Auto (English)", "อัตโนมัติ (ภาษาไทย)");
    m.insert("at", "ที่");
    m.insert("Resets in", "รีเซ็ตใน");
    m.insert("Tray icon colors:", "สีไอคอนเทรย์:");
    m.insert("< 50% usage", "< 50% การใช้งาน");
    m.insert("50-79% usage", "50\u{2013}79% การใช้งาน");
    m.insert(">= 80% usage", "\u{2265} 80% การใช้งาน");
    m.insert("No data", "ไม่มีข้อมูล");
    m.insert("exceeded", "เกินกำหนด");
    m.insert("Show widget", "แสดงวิดเจ็ต");
    m.insert("Check for updates", "ตรวจสอบอัปเดต");
    m.insert("Accessibility patterns", "รูปแบบการเข้าถึง");
    m.insert("Update available", "มีอัปเดตใหม่");
    m.insert(
        "is available. Click to download.",
        "พร้อมใช้งาน คลิกเพื่อดาวน์โหลด",
    );
    m.insert("Icon style", "รูปแบบไอคอน");
    m.insert("Number", "ตัวเลข");
    m.insert("Ring", "วงแหวน");
    m.insert("Bar", "แถบ");
    m.insert("Pie", "วงกลม");
    m.insert("Dashboard layout", "รูปแบบแดชบอร์ด");
    m.insert("Minimal", "น้อยที่สุด");
    m.insert("Standard", "มาตรฐาน");
    m.insert("Detailed", "ละเอียด");
    m.insert("Hide Extra Usage", "ซ่อน Extra Usage");
    m.insert("stale_token_expired", "ข้อมูลเก่า — โทเคนหมดอายุ");
    m.insert("stale_data", "ข้อมูลเก่า — ค่าที่ทราบล่าสุด");
    m.insert("codex_no_data", "ไม่มีหน้าต่างลิมิตของ Codex ที่ใช้งานอยู่ ข้อมูลการใช้งานอ่านจากบันทึกเซสชัน Codex ในเครื่อง — เริ่มหรือทำเซสชันต่อ แล้วข้อมูลจะปรากฏที่นี่");
    m.insert("Open Codex Usage →", "เปิดการใช้งาน Codex →");
    m.insert("Midnight", "เที่ยงคืน");
    m.insert("Sunset", "พระอาทิตย์ตก");
    m.insert("Show Codex section", "แสดงส่วน Codex");
    m.insert("Reopen the tray popup to refresh", "เปิดหน้าต่างใหม่เพื่อรีเฟรช");
    m.insert("Show startup notification", "แสดงการแจ้งเตือนเมื่อเริ่มทำงาน");
    m.insert("Show login expiry warning", "เตือนเมื่อการเข้าสู่ระบบหมดอายุ");
    m.insert("Alert thresholds", "เกณฑ์การแจ้งเตือน");
    m.insert("Notification sound", "เสียงแจ้งเตือน");
    m.insert("Test notification", "ทดสอบการแจ้งเตือน");
    m.insert("Send", "ส่ง");
    m
}

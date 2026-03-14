pub const APP_TITLE: &str = "人脸识别预警系统";
pub const APP_STYLES: &str = r#"
    :root {
        color-scheme: light;
        --bg: #f3efe6;
        --panel: rgba(255, 252, 246, 0.92);
        --panel-muted: rgba(224, 214, 196, 0.55);
        --line: rgba(79, 54, 38, 0.14);
        --text: #2e241d;
        --muted: #756658;
        --accent: #a43e2f;
        --accent-strong: #7f281c;
    }

    body {
        margin: 0;
        font-family: "Microsoft YaHei UI", "Noto Sans SC", sans-serif;
        background:
            radial-gradient(circle at top right, rgba(164, 62, 47, 0.16), transparent 28%),
            linear-gradient(180deg, #efe7da 0%, #f8f3eb 46%, #efe7da 100%);
        color: var(--text);
    }

    .app-shell {
        min-height: 100vh;
        padding: 24px;
        display: flex;
        flex-direction: column;
        gap: 18px;
    }

    .app-header,
    .panel {
        border: 1px solid var(--line);
        border-radius: 24px;
        background: var(--panel);
        backdrop-filter: blur(10px);
        box-shadow: 0 18px 50px rgba(67, 44, 30, 0.08);
    }

    .app-header {
        padding: 24px;
    }

    .eyebrow {
        margin: 0 0 10px;
        color: var(--accent);
        font-size: 12px;
        letter-spacing: 0.18em;
        text-transform: uppercase;
    }

    h1, h2, p {
        margin: 0;
    }

    .subtitle,
    .hint,
    .meta,
    .empty-state {
        color: var(--muted);
    }

    .app-content {
        display: grid;
        gap: 18px;
    }

    .page {
        display: grid;
        gap: 14px;
    }

    .page-title {
        display: grid;
        gap: 6px;
    }

    .section-header,
    .action-row,
    .list-item-top {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
    }

    .banner {
        margin-top: 12px;
        padding: 12px 14px;
        border-radius: 14px;
        font-size: 14px;
    }

    .banner.success {
        background: rgba(51, 112, 69, 0.12);
        color: #265b37;
    }

    .banner.error {
        background: rgba(164, 62, 47, 0.12);
        color: #8c3023;
    }

    .panel {
        padding: 20px;
    }

    .panel.muted {
        background: var(--panel-muted);
    }

    .tab-bar {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 12px;
    }

    .tab-button {
        border: none;
        border-radius: 999px;
        padding: 14px 16px;
        background: rgba(255, 250, 241, 0.76);
        color: var(--text);
        font-size: 15px;
    }

    .tab-button.active {
        background: linear-gradient(135deg, var(--accent), var(--accent-strong));
        color: #fff9f5;
    }

    .primary-button,
    .ghost-button {
        border-radius: 999px;
        padding: 10px 14px;
        font-size: 14px;
        cursor: pointer;
    }

    .primary-button {
        border: none;
        background: linear-gradient(135deg, var(--accent), var(--accent-strong));
        color: #fff9f5;
    }

    .ghost-button {
        border: 1px solid var(--line);
        background: rgba(255, 255, 255, 0.72);
        color: var(--text);
    }

    .ghost-button.danger {
        color: #8c3023;
    }

    .list {
        list-style: none;
        margin: 0;
        padding: 0;
        display: grid;
        gap: 10px;
    }

    .list-item {
        display: grid;
        gap: 4px;
        padding: 14px 16px;
        border-radius: 16px;
        background: rgba(255, 255, 255, 0.65);
        border: 1px solid rgba(79, 54, 38, 0.1);
    }

    .form-grid {
        display: grid;
        gap: 12px;
        margin-top: 12px;
    }

    label {
        display: grid;
        gap: 6px;
    }

    input {
        border: 1px solid var(--line);
        border-radius: 12px;
        padding: 12px 14px;
        background: rgba(255, 255, 255, 0.82);
    }

    textarea {
        min-height: 120px;
        border: 1px solid var(--line);
        border-radius: 12px;
        padding: 12px 14px;
        background: rgba(255, 255, 255, 0.82);
        resize: vertical;
    }

    .overlay {
        position: fixed;
        inset: 0;
        background: rgba(46, 36, 29, 0.36);
        display: grid;
        place-items: center;
        padding: 24px;
    }

    .modal-panel,
    .viewer-panel {
        width: min(720px, 100%);
    }

    .preview-image {
        width: 100%;
        max-height: 240px;
        object-fit: contain;
        border-radius: 16px;
        margin-top: 12px;
    }
"#;

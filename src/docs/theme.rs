pub const EMERALD_DARK_CSS: &str = r#"
/* ==========================================================================
   AgriTech OpenAPI Swagger UI Theme (Dark Emerald Pro - Eye Comfort Edition)
   Anti-Glare, Ergonomic Dark Contrast for Professional Developer Portals
   ========================================================================== */

/* 1. Global Reset, Typography & Canvas */
body {
  background-color: #0B1121 !important;
  color: #E2E8F0 !important;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif !important;
  margin: 0 !important;
  padding: 0 !important;
}

.swagger-ui {
  color: #E2E8F0 !important;
}

/* Custom Eye-Friendly Dark Scrollbars */
.swagger-ui *::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}
.swagger-ui *::-webkit-scrollbar-track {
  background: #0B1121;
}
.swagger-ui *::-webkit-scrollbar-thumb {
  background: #1E293B;
  border-radius: 4px;
}
.swagger-ui *::-webkit-scrollbar-thumb:hover {
  background: #334155;
}

/* 2. Top Navigation Bar */
.swagger-ui .topbar {
  background-color: #111827 !important;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08) !important;
  padding: 12px 0 !important;
}

.swagger-ui .topbar a {
  color: #10B981 !important;
  font-weight: 700 !important;
}

.swagger-ui .topbar .download-url-wrapper .select-label select {
  border: 1px solid rgba(255, 255, 255, 0.15) !important;
  background-color: #1F2937 !important;
  color: #F8FAFC !important;
}

/* 3. API Info & Description Header */
.swagger-ui .info {
  margin: 28px 0 20px 0 !important;
}

.swagger-ui .info .title {
  color: #F8FAFC !important;
  font-weight: 700 !important;
  letter-spacing: -0.025em !important;
}

.swagger-ui .info .title small {
  background-color: rgba(16, 185, 129, 0.15) !important;
  color: #34D399 !important;
  border: 1px solid rgba(16, 185, 129, 0.3) !important;
  border-radius: 12px !important;
  padding: 2px 10px !important;
  font-size: 12px !important;
}

.swagger-ui .info p,
.swagger-ui .info li,
.swagger-ui .info table {
  color: #94A3B8 !important;
  font-size: 14px !important;
  line-height: 1.6 !important;
}

.swagger-ui .info a {
  color: #38BDF8 !important;
  text-decoration: none !important;
}
.swagger-ui .info a:hover {
  text-decoration: underline !important;
}

/* 4. Scheme & Server Container */
.swagger-ui .scheme-container {
  background-color: #141A28 !important;
  box-shadow: none !important;
  border-radius: 8px !important;
  border: 1px solid rgba(255, 255, 255, 0.08) !important;
  padding: 14px 20px !important;
  margin-bottom: 24px !important;
}

.swagger-ui .schemes-title {
  color: #E2E8F0 !important;
}

.swagger-ui select {
  background-color: #0F172A !important;
  color: #F8FAFC !important;
  border: 1px solid rgba(255, 255, 255, 0.12) !important;
  border-radius: 6px !important;
  padding: 6px 12px !important;
  outline: none !important;
}
.swagger-ui select:focus {
  border-color: #10B981 !important;
}

/* 5. Tag Section Headers */
.swagger-ui .opblock-tag {
  color: #F8FAFC !important;
  font-size: 18px !important;
  font-weight: 600 !important;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1) !important;
  padding: 16px 0 10px 0 !important;
  margin: 20px 0 12px 0 !important;
}

.swagger-ui .opblock-tag small {
  color: #64748B !important;
  font-size: 13px !important;
  font-weight: 400 !important;
  margin-left: 10px !important;
}

.swagger-ui .opblock-tag:hover {
  background: transparent !important;
}

/* 6. Opblock Cards (Endpoint Containers) */
.swagger-ui .opblock {
  background-color: #141A28 !important;
  border-radius: 8px !important;
  border: 1px solid rgba(255, 255, 255, 0.08) !important;
  margin: 0 0 12px !important;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.25) !important;
  overflow: hidden !important;
}

.swagger-ui .opblock-summary {
  padding: 10px 16px !important;
  border-bottom: 1px solid transparent !important;
}

.swagger-ui .opblock.is-open .opblock-summary {
  border-bottom: 1px solid rgba(255, 255, 255, 0.06) !important;
}

.swagger-ui .opblock .opblock-summary-path,
.swagger-ui .opblock .opblock-summary-path__deprecated {
  color: #F8FAFC !important;
  font-size: 14px !important;
  font-weight: 500 !important;
}

.swagger-ui .opblock .opblock-summary-description {
  color: #94A3B8 !important;
  font-size: 13px !important;
}

/* 7. HTTP Method Badges - High Legibility */
.swagger-ui .opblock .opblock-summary-method {
  min-width: 72px !important;
  text-align: center !important;
  border-radius: 6px !important;
  font-weight: 700 !important;
  font-size: 12px !important;
  text-shadow: none !important;
  padding: 5px 10px !important;
}

.swagger-ui .opblock.opblock-get {
  border-color: rgba(2, 132, 199, 0.3) !important;
  background: #111A2E !important;
}
.swagger-ui .opblock.opblock-get .opblock-summary-method {
  background: #0284C7 !important;
  color: #FFFFFF !important;
}

.swagger-ui .opblock.opblock-post {
  border-color: rgba(5, 150, 105, 0.3) !important;
  background: #0F1E24 !important;
}
.swagger-ui .opblock.opblock-post .opblock-summary-method {
  background: #059669 !important;
  color: #FFFFFF !important;
}

.swagger-ui .opblock.opblock-put,
.swagger-ui .opblock.opblock-patch {
  border-color: rgba(217, 119, 6, 0.3) !important;
  background: #1C1B1E !important;
}
.swagger-ui .opblock.opblock-put .opblock-summary-method,
.swagger-ui .opblock.opblock-patch .opblock-summary-method {
  background: #D97706 !important;
  color: #FFFFFF !important;
}

.swagger-ui .opblock.opblock-delete {
  border-color: rgba(220, 38, 38, 0.3) !important;
  background: #1F151B !important;
}
.swagger-ui .opblock.opblock-delete .opblock-summary-method {
  background: #DC2626 !important;
  color: #FFFFFF !important;
}

/* 8. ELIMINATE GLARE: Opblock Internal Section Headers & Body */
.swagger-ui .opblock-body {
  background-color: #0F1626 !important;
  padding: 16px 20px !important;
}

/* Fix stark white banners for "Parameters" and "Responses" */
.swagger-ui .opblock .opblock-section-header {
  background-color: #172136 !important;
  box-shadow: none !important;
  border: 1px solid rgba(255, 255, 255, 0.06) !important;
  border-radius: 6px !important;
  padding: 8px 14px !important;
  margin: 12px 0 10px 0 !important;
}

.swagger-ui .opblock .opblock-section-header h4 {
  color: #E2E8F0 !important;
  font-size: 13px !important;
  font-weight: 600 !important;
  letter-spacing: 0.02em !important;
}

.swagger-ui .opblock-description-wrapper p,
.swagger-ui .opblock-external-docs-wrapper p,
.swagger-ui .opblock-title_normal p {
  color: #94A3B8 !important;
  font-size: 13px !important;
}

/* 9. ELIMINATE GLARE: Dark Request Body Textarea & Syntax Boxes */
.swagger-ui .body-param__text {
  background-color: #080D1A !important;
  color: #38BDF8 !important;
  border: 1px solid rgba(255, 255, 255, 0.12) !important;
  border-radius: 6px !important;
  font-family: 'JetBrains Mono', 'Fira Code', 'Courier New', monospace !important;
  font-size: 13px !important;
  line-height: 1.5 !important;
  padding: 12px !important;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.3) !important;
}

.swagger-ui .body-param__text:focus {
  border-color: #10B981 !important;
  outline: none !important;
  box-shadow: 0 0 0 2px rgba(16, 185, 129, 0.25) !important;
}

.swagger-ui .highlight-code,
.swagger-ui .microlight,
.swagger-ui pre {
  background-color: #080D1A !important;
  color: #38BDF8 !important;
  border-radius: 6px !important;
  border: 1px solid rgba(255, 255, 255, 0.08) !important;
  font-family: 'JetBrains Mono', 'Fira Code', monospace !important;
  font-size: 12.5px !important;
}

.swagger-ui .example__item,
.swagger-ui .example__item pre {
  background-color: #080D1A !important;
  color: #CBD5E1 !important;
}

/* 10. Refined Interactive Buttons (Non-Aggressive Emerald) */
.swagger-ui .btn.authorize {
  color: #10B981 !important;
  border: 1px solid #10B981 !important;
  background: rgba(16, 185, 129, 0.08) !important;
  border-radius: 6px !important;
  font-weight: 600 !important;
  padding: 6px 16px !important;
  transition: all 0.2s ease !important;
}

.swagger-ui .btn.authorize:hover {
  background: #10B981 !important;
  color: #0B1121 !important;
}

.swagger-ui .btn.authorize svg {
  fill: #10B981 !important;
}
.swagger-ui .btn.authorize:hover svg {
  fill: #0B1121 !important;
}

/* Refined Execute CTA Button */
.swagger-ui .btn.execute {
  background: linear-gradient(135deg, #059669 0%, #10B981 100%) !important;
  border: none !important;
  color: #FFFFFF !important;
  font-weight: 600 !important;
  border-radius: 6px !important;
  padding: 10px 24px !important;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.25) !important;
  transition: all 0.2s ease !important;
  letter-spacing: 0.02em !important;
}

.swagger-ui .btn.execute:hover {
  background: linear-gradient(135deg, #047857 0%, #059669 100%) !important;
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3) !important;
  transform: translateY(-1px);
}

.swagger-ui .btn.try-out__btn {
  background: rgba(255, 255, 255, 0.05) !important;
  color: #38BDF8 !important;
  border: 1px solid rgba(56, 189, 248, 0.3) !important;
  border-radius: 6px !important;
  font-weight: 500 !important;
}
.swagger-ui .btn.try-out__btn:hover {
  background: rgba(56, 189, 248, 0.15) !important;
}

.swagger-ui .btn.cancel {
  border: 1px solid rgba(239, 68, 68, 0.35) !important;
  color: #F87171 !important;
  background: rgba(239, 68, 68, 0.08) !important;
  border-radius: 6px !important;
}
.swagger-ui .btn.cancel:hover {
  background: rgba(239, 68, 68, 0.2) !important;
}

/* 11. Tables, Parameters & Responses */
.swagger-ui table.parameters,
.swagger-ui table.responses-table {
  background-color: transparent !important;
}

.swagger-ui table.parameters th,
.swagger-ui table.responses-table th {
  color: #94A3B8 !important;
  font-size: 12px !important;
  text-transform: uppercase !important;
  letter-spacing: 0.05em !important;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08) !important;
  padding: 10px 8px !important;
}

.swagger-ui table.parameters td,
.swagger-ui table.responses-table td {
  color: #E2E8F0 !important;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04) !important;
  padding: 10px 8px !important;
}

.swagger-ui .parameter__name {
  color: #F8FAFC !important;
  font-weight: 600 !important;
}
.swagger-ui .parameter__name.required:after {
  color: #EF4444 !important;
}
.swagger-ui .parameter__type {
  color: #38BDF8 !important;
  font-family: 'JetBrains Mono', monospace !important;
  font-size: 12px !important;
}
.swagger-ui .parameter__in {
  color: #64748B !important;
  font-style: italic !important;
  font-size: 12px !important;
}

.swagger-ui .response-col_status {
  color: #34D399 !important;
  font-family: 'JetBrains Mono', monospace !important;
  font-weight: 700 !important;
}
.swagger-ui .response-col_description__inner div.markdown p {
  color: #CBD5E1 !important;
}

.swagger-ui .responses-inner {
  background: transparent !important;
}

/* 12. Models (Schemas) Accordion at Page Bottom */
.swagger-ui section.models {
  background-color: #141A28 !important;
  border: 1px solid rgba(255, 255, 255, 0.08) !important;
  border-radius: 8px !important;
  margin: 24px 0 !important;
}

.swagger-ui section.models h4 {
  color: #F8FAFC !important;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06) !important;
}

.swagger-ui section.models .model-container {
  background-color: #0F1626 !important;
  border-radius: 6px !important;
  margin: 8px !important;
  padding: 10px !important;
}

.swagger-ui .model-box {
  background-color: #080D1A !important;
  color: #CBD5E1 !important;
}

.swagger-ui .model-title {
  color: #F8FAFC !important;
}

.swagger-ui .prop-type {
  color: #38BDF8 !important;
}

.swagger-ui .prop-format {
  color: #64748B !important;
}

/* 13. Authorize Dialog Modal */
.swagger-ui .dialog-ux .backdrop-ux {
  background-color: rgba(3, 7, 18, 0.8) !important;
  backdrop-filter: blur(4px) !important;
}

.swagger-ui .modal-ux {
  background-color: #141A28 !important;
  border: 1px solid rgba(255, 255, 255, 0.12) !important;
  border-radius: 12px !important;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.6) !important;
  color: #F8FAFC !important;
}

.swagger-ui .modal-ux-header {
  border-bottom: 1px solid rgba(255, 255, 255, 0.08) !important;
  padding: 16px 20px !important;
}
.swagger-ui .modal-ux-header h3 {
  color: #F8FAFC !important;
  font-weight: 700 !important;
}

.swagger-ui .modal-ux-content {
  padding: 20px !important;
}
.swagger-ui .modal-ux-content h4 {
  color: #E2E8F0 !important;
}
.swagger-ui .modal-ux-content p {
  color: #94A3B8 !important;
}

.swagger-ui .modal-ux input[type="text"],
.swagger-ui .modal-ux input[type="password"] {
  background-color: #080D1A !important;
  border: 1px solid rgba(255, 255, 255, 0.15) !important;
  border-radius: 6px !important;
  color: #38BDF8 !important;
  font-family: 'JetBrains Mono', monospace !important;
  padding: 8px 12px !important;
}
"#;

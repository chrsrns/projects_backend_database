# Resume Markdown Format

This document describes the Markdown format used by the resume import/export API. A file that follows this format can be imported via `POST /api/resume/import/markdown` or `POST /api/resume/{id}/import/markdown` and exported via `GET /api/resume/{id}/export/markdown`.

## Front matter (optional)

An exported resume opens with a `---`-fenced front-matter block. On import the block picks the target row and carries variant targeting metadata; it is stripped before the body is parsed, so it never reaches resume fields or `ResumeDocument`.

```markdown
---
resume_id: 42
company_name: "Acme Corp"
role_title: "Staff Backend Engineer"
target_date: "2026-03"
job_description: "Own the resume pipeline"
variant_label: "acme-tailoring"
show_variant_tag: true
---
# Jane Doe
```

- **Placement**: the first non-blank line of the file must be `---` (trailing whitespace allowed). An optional UTF-8 BOM and leading blank lines are tolerated. If the first non-blank line is not `---`, the file parses without a front-matter block.
- **Lines**: one `key: <JSON scalar>` or bare `key:` per line; leading whitespace and whitespace around the colon are tolerated. A bare `key:` and `key: null` are equivalent.
- **Keys**: `resume_id`, `company_name`, `role_title`, `target_date`, `job_description`, `variant_label`, `show_variant_tag`. Values must be JSON scalars — strings are double-quoted and JSON-escaped, `show_variant_tag` is `true`/`false`, `resume_id` is an integer or numeric string.
- **Rejected with `400`**: unknown or duplicate keys, a line without a colon, a blank line inside the block, a non-scalar value, and an unclosed fence. A file that opens with `---` never degrades to body text — a legacy file whose first non-blank line is a `---` thematic break must remove it or make the block well formed.
- **`resume_id`**: routing marker, must be a positive integer. `resume_id:` empty, `null`, or a non-integer value → `400`. Absent/foreign row → `404`/`403`; the marker never falls back to the email match.
- **Variant metadata keys** (`company_name`, `role_title`, `target_date`, `job_description`, `variant_label`, `show_variant_tag`): applied only when the import target is a variant. On a variant target: absent key = no change, empty/`null` = stored `NULL` (`show_variant_tag` rejects empty/`null` instead — it is a required boolean), strings obey the usual caps, `target_date` is an ISO partial date (`YYYY`, `YYYY-MM`, `YYYY-MM-DD`). Metadata keys on a base resume target or on the create path → `400`.
- **Export**: always emits `resume_id`. A variant export also emits every stored non-null metadata key; `show_variant_tag` is emitted only when the exporter owns the resume.
- **Variant target quirk**: on a variant the markdown `Email` and `Public` bullets are still required/validated but ignored — the stored `email`, `is_public`, and `created_by` win.
- **Explicit route**: `POST /api/resume/{id}/import/markdown` targets the URL id directly (owner only). A front-matter `resume_id` that disagrees with the URL id → `400`.
- **Validate/convert**: `POST /api/resume/validate/markdown` and `POST /api/resume/convert/markdown` parse the block and ignore it; a malformed block reports invalid or answers `400`.

## Top-level header (H1)

The first `# ` heading is the resume owner's name.

```markdown
# Jane Doe
```

## Header bullets

Immediately after the H1, write a list of header fields. `Email` is required. All other fields are optional.

```markdown
- Location: New York, NY
- Email: jane.doe@example.com
- Profile Image: https://example.com/photo.jpg
- GitHub: https://github.com/janedoe
- Mobile: +1987654321
- Video: https://example.com/video.mp4
- Public: true
```

- `Location`: free text.
- `Email`: must contain `@` and a domain with a `.`.
- `Profile Image`: URL.
- `GitHub`: URL.
- `Mobile`: free text.
- `Video`: URL or free text. Limited to 500 characters. Markdown link form is not supported and is ignored.
- `Public`: `true`, `yes`, or `1` for public; anything else is private.

URLs in header bullets may be written as plain text or as Markdown links. They are stored and exported as plain text. The `Video` bullet only accepts plain text; a Markdown link in a `Video` bullet is ignored and the video is stored as `NULL`.

## Summary (optional)

If present, the `## Summary` section is imported as the resume's executive summary.

```markdown
## Summary

Systems engineer with distributed-systems experience.
```

- The summary is limited to 5,000 characters.
- An empty or whitespace-only summary is treated as absent and stored as `NULL`.
- On export, the section is omitted when the executive summary is `NULL`.

## Education

```markdown
## Education

### Bachelor's in Computer Science - University of ABC (Sep 2020 - May 2024) [order: 0]
- Degree: Bachelor of Science
- Description: Focused on software engineering
- Graduated with honors
- Specialized in distributed systems
```

- H3 heading format: `### <Education Stage> - <Institution Name> (<Start Date> - <End Date>) [order: <number>]`
- The ` - ` separator between stage and institution is required.
- The final parenthesized segment must be a valid date range (see [Dates](#dates) for syntax).
- If the institution name itself contains parentheses, the last parenthesized segment is still parsed as the date range (e.g. `University of (ABC) (Sep 2020 - May 2024)`).
- `[order: <number>]` is optional and controls display order.
- Bullets:
  - `Degree:` sets the degree field.
  - `Description:` sets the description field.
  - All other bullets are education key points.

## Skills

```markdown
## Skills

- Rust - 90% [order: 0]
- Python - 75% [order: 1]
- JavaScript - 60% [order: 2]
```

- Bullet format: `- <Skill Name> - <percentage>% [order: <number>]`
- Percentage must be an integer between `0` and `100` inclusive.
- `[order: <number>]` is optional.

## Work Experience

```markdown
## Work Experience

### Senior Software Engineer - Tech Corp (Jan 2020 - Present) [order: 0]
- Description: Backend development
- Led team of 5 developers
- Improved system performance by 40%
```

- H3 heading format: `### <Job Title> - <Company Name> (<Start Date> - <End Date>) [order: <number>]`
- The company name and its ` - ` separator are optional.
- The final parenthesized segment must be a valid date range (see [Dates](#dates) for syntax).
- `[order: <number>]` is optional.
- Bullets:
  - `Description:` sets the description field.
  - All other bullets are work-experience key points.

## Portfolio Projects

```markdown
## Portfolio Projects

### My Portfolio [order: 0]
- Live: https://example.com
- Source: https://github.com/janedoe/project
- Image: https://example.com/project.png
- Technologies: Rust, Rocket, Diesel
- Description: Built a scalable resume API
- Deployed to production
```

- H3 heading format: `### <Project Name> [order: <number>]`
- `[order: <number>]` is optional.
- Bullets:
  - `Live:` project link URL. Limited to 500 characters. An empty or whitespace-only value is stored as `NULL`. A Markdown link is flattened to `text (url)` and stored as plain text.
  - `Source:` source-code link URL. Limited to 500 characters. An empty or whitespace-only value is stored as `NULL`. A Markdown link is flattened to `text (url)` and stored as plain text.
  - `Image:` image URL. Limited to 500 characters. An empty or whitespace-only value is stored as `NULL`. A Markdown link is flattened to `text (url)` and stored as plain text.
  - `Video:` video URL or free text. Limited to 500 characters. Markdown link form is not supported and is ignored.
  - `Technologies:` comma-separated list of technology names.
  - `Description:` sets the description field.
  - All other bullets are project key points.

## Languages & Frameworks

```markdown
## Languages & Frameworks

### Rust [order: 0]
- Rocket
- Actix

### Python [order: 1]
- Django
- FastAPI
```

- H3 heading format: `### <Language Name> [order: <number>]`
- `[order: <number>]` is optional.
- Bullets under each language are framework names.

## Dates

In `Education` and `Work Experience` headings, the **final parenthesized segment** is parsed as the date range. A date range must be written as `(<Start> - <End>)` and must contain at least one hyphen.

- Start and end dates may use one of these formats:
  - `2020` — four-digit year.
  - `Sep 2020` — three-letter English month abbreviation followed by a four-digit year (case-insensitive).
  - `September 2020` — full English month name followed by a four-digit year (case-insensitive).
  - `2020-09-01` — ISO `YYYY-MM-DD`.
- The end date may be `Present`, `Current`, or `Ongoing` (case-insensitive) instead of a real date.
- The start date must not be after the end date.
- Text in parentheses without a hyphen is treated as part of the title, not a date range.

Examples of valid date ranges:

```markdown
(2020 - 2024)
(Sep 2020 - May 2024)
(September 2020 - May 2024)
(2020-09-01 - 2024-05-01)
(Jan 2020 - Present)
(January 1919 - Present)
```

Invalid date ranges:

- `(Sep 2020 to May 2024)` — missing hyphen.
- `(2020-09 - 2024-05)` — ISO month-only `YYYY-MM` is not supported in Markdown.
- `(ABC)` — no hyphen, so it is treated as title text.

## Display order

Append `[order: <integer>]` to any H3 heading or skill bullet to override display order. Lower numbers appear first. If omitted, items appear in document order.

## Validation limits

- Markdown import payload must be ≤ 1 MiB.
- `Email` is required and must pass basic validation.
- Unknown H2 section names are rejected. Valid names are: `Summary`, `Education`, `Skills`, `Work Experience`, `Portfolio Projects`, `Languages & Frameworks`.
- Skill percentages must be in the range `[0, 100]`.
- Executive summary is limited to 5,000 characters.
- Video is limited to 500 characters.
- Portfolio project `Live:`, `Source:`, `Image:`, and `Video:` are limited to 500 characters.
- Empty or whitespace-only `Live:`, `Source:`, and `Image:` bullets are stored as `NULL`.

## Full example

```markdown
# Jane Doe

- Location: New York, NY
- Email: jane.doe@example.com
- GitHub: https://github.com/janedoe
- Mobile: +1987654321
- Video: https://example.com/video.mp4
- Public: true

## Summary

Systems engineer with distributed-systems experience.

## Education

### Bachelor's in Computer Science - University of ABC (Sep 2020 - May 2024)
- Degree: Bachelor of Science
- Description: Focused on software engineering
- Graduated with honors
- Specialized in distributed systems

## Skills

- Rust - 90%
- Python - 75%
- JavaScript - 60%

## Work Experience

### Senior Software Engineer - Tech Corp (Jan 2020 - Present)
- Description: Backend development
- Led team of 5 developers
- Improved system performance by 40%

## Portfolio Projects

### My Portfolio
- Live: https://example.com
- Source: https://github.com/janedoe/project
- Technologies: Rust, Rocket, Diesel
- Built a scalable resume API

## Languages & Frameworks

### Rust
- Rocket
- Actix

### Python
- Django
- FastAPI
```

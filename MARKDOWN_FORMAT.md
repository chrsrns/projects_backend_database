# Resume Markdown Format

This document describes the Markdown format used by the resume import/export API. A file that follows this format can be imported via `POST /api/resume/import/markdown` and exported via `GET /api/resume/{id}/export/markdown`.

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
- Public: true
```

- `Location`: free text.
- `Email`: must contain `@` and a domain with a `.`.
- `Profile Image`: URL.
- `GitHub`: URL.
- `Mobile`: free text.
- `Public`: `true`, `yes`, or `1` for public; anything else is private.

URLs may be written as plain text or as Markdown links. They are stored and exported as plain text.

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
- The final parenthesized segment must be a valid date range.
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
- The final parenthesized segment must be a valid date range.
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
  - `Live:` project link URL.
  - `Source:` source-code link URL.
  - `Image:` image URL.
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

Date ranges use the format `(<Start> - <End>)`.

- Start and end dates can be written as:
  - `Sep 2020` (`%b %Y`)
  - `2020-09-01` (`%Y-%m-%d`)
  - `<Month Abbreviation> <Year>` such as `January 2020` or `Jan 2020`
- The end date can be `Present`, `Current`, or `Ongoing`.
- The start date must not be after the end date.

## Display order

Append `[order: <integer>]` to any H3 heading or skill bullet to override display order. Lower numbers appear first. If omitted, items appear in document order.

## Validation limits

- Markdown import payload must be ≤ 1 MiB.
- `Email` is required and must pass basic validation.
- Unknown H2 section names are rejected. Valid names are: `Summary`, `Education`, `Skills`, `Work Experience`, `Portfolio Projects`, `Languages & Frameworks`.
- Skill percentages must be in the range `[0, 100]`.
- Executive summary is limited to 5,000 characters.

## Full example

```markdown
# Jane Doe

- Location: New York, NY
- Email: jane.doe@example.com
- GitHub: https://github.com/janedoe
- Mobile: +1987654321
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

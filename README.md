# Autospieler

## Environment Variables

- `DAUERZUSAGE_EMAIL` and `DAUERZUSAGE_PASSWORT`: Spielerplus Benutzername und Passwort
- DAUERZUSAGE_ID: Die ID, mit der euch Spielerplus intern eurem Team zuordnet. Diese findet ihr heraus, indem ihr ganz oben links auf euren Namen/Team klickt. Ihr landet dann auf der "Team auswählen" Seite. Der Link zu eurem Team hat das Format `https://www.spielerplus.de/site/switch-user?id=<DAUERZUSAGE_ID>`, ihr könnt also dort die ID auslesen.
- `OUTLOOK_CALENDAR_ID`: The ID of the Calendar (can be found using Graph API)
- `OUTLOOK_USER_PRINCIPAL_NAME`: The mailbox email or your own user email that will be the organizer for all events created by Autospieler.
- `ENTRA_CLIENT_ID`, `ENTRA_CLIENT_SECRET` and `ENTRA_TENANT_ID`: Credentials from the Entra application that you'll need to create for Autospieler. Application permissions for `Calendar.ReadWrite` is required. If you want to restrict the token from accessing other UPNs Calendars, follow the guide here: [https://learn.microsoft.com/en-us/graph/auth-limit-mailbox-access]([https://learn.microsoft.com/en-us/graph/auth-limit-mailbox-access])

Credits:

- The scraping code is based on DrTobe's work: [https://github.com/DrTobe/dauerzusagesendung](https://github.com/DrTobe/dauerzusagesendung).

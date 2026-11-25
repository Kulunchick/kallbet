# Telegram as the only sign-in method

Users sign in with the Telegram Login Widget. The friend groups already use Telegram, we store no passwords, and the same bot sends reminders. Steam OpenID was the alternative. The widget only works on the domain registered with the bot through BotFather `/setdomain`, so local development needs a tunnel or a dev-only login bypass.

## Amendment: Bot Users (spec #21)

_Contradicts the rule above for non-human Users only, reopened intentionally._ Humans still sign in with the Telegram Login Widget and nothing else; no password or other browser login is added. A Bot User cannot use the widget, so it authenticates machine-to-machine: a sidecar that holds a secret from the server's environment (`KALLBET_BOT_SECRET`) registers each Bot User at a dedicated endpoint and gets a Bearer token per bot, which the server resolves to that Bot User the same way a session resolves to a human. Without the secret configured the endpoint is off, so an instance that runs no bots exposes no machine sign-in.

### Consequences

- A Bot User has no Telegram account: its Telegram notifications are off and nothing is ever sent to it.
- Whoever holds the shared secret can act as any Bot User, and only as a Bot User: the secret itself is no User, and a token never reaches a human's account.

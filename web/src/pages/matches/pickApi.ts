import { ApiError } from "../../api/client";

/**
 * Saves a pick without the shared "Prediction saved" notice that `api()` raises
 * for every mutation: the page shows its own confirmation ("Pick saved · team",
 * with Undo) and keeps errors inline under the card.
 *
 * Same endpoint and errors as `predict` in `./api`. Drop this file once `api()`
 * can be told to stay quiet.
 */
export async function putPick(league: number, match: number, team: number): Promise<void> {
  let response: Response;
  try {
    response = await fetch(`/api/leagues/${league}/matches/${match}/prediction`, {
      method: "PUT",
      credentials: "same-origin",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ team_id: team }),
    });
  } catch {
    throw new ApiError(0, "network");
  }
  if (!response.ok) {
    const payload = await response.json().catch(() => null);
    throw new ApiError(response.status, payload?.error?.code ?? "internal");
  }
}

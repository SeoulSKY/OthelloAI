import express from "express";
import pino from "pino";
import path from "path";
import fs from "fs";
import { duration } from "moment";
import cron from "node-cron";

import stream from "stream";
import util from "util";

const pipeline = util.promisify(stream.pipeline);

const IS_PRODUCTION = process.env.PRODUCTION !== undefined;
const IS_DOCKER = process.env.DOCKER !== undefined;
const AI_SERVER_HOST = IS_PRODUCTION ? "http://ai-server:8000/api" : "http://localhost:8000/api";
const HOST = "0.0.0.0";
const PORT = 8080;
const RETRY_INTERVAL = duration({second: 5});

export const logger = pino({
    transport: {
        target: "pino-pretty"
    },
})

export const PROJECT_ROOT_PATH = path.dirname(require.main?.path as string);


const app = express();

let latestTag: string | null = null;

async function fetchLatestTag() {
    let response = await fetch(
        "https://api.github.com/repos/seoulsky/desdemona/releases/latest"
    );
    if (!response.ok) {
        logger.error("Failed to fetch latest release info");
        logger.error(await response.text());
        return latestTag; // return previous tag
    }

    let json = await response.json();
    return json.tag_name;
}

if (require.main === module) {
    (async () => {
        if (!IS_DOCKER && !fs.existsSync(path.join(PROJECT_ROOT_PATH, "public", "Build"))) {
            logger.error("Unity build not found. Run `npm run build-unity` first.");
            process.exit(1);
        }

        if (IS_DOCKER) {
            latestTag = (await fetchLatestTag()) ?? process.exit(1);

            // run for every 5 minute
            cron.schedule("*/5 * * * *", async () => {
                latestTag = await fetchLatestTag();
            });

            app.get("/Build/:filename", async (req, res) => {
                let response = await fetch(
                    `https://github.com/SeoulSKY/Desdemona/releases/download/${latestTag}/${req.params.filename}`
                );

                if (!response.ok) {
                    res.status(response.status).send(
                        await response.text()
                    );
                    return;
                }

                res.setHeader("Access-Control-Allow-Origin", "*");
                res.setHeader(
                    "Content-Type",
                    response.headers.get("Content-Type")!
                );
                res.setHeader(
                    "Content-Length",
                    response.headers.get("Content-Length")!
                );

                res.setHeader("Cache-Control", "no-transform");

                if (req.params.filename.endsWith(".unityweb")) {
                    res.setHeader("Content-Encoding", "gzip");
                }

                if (req.params.filename.includes(".wasm")) {
                    res.setHeader("Content-Type", "application/wasm");
                } else if (req.params.filename.includes(".js")) {
                    res.setHeader(
                        "Content-Type",
                        "application/javascript"
                    );
                }

                await pipeline(response.body as any, res).catch(e => {
                    if (res.closed) {
                        return;
                    }

                    logger.error(e);
                    res.status(500).send("Internal Server Error");
                });
            });
        }

        if (IS_PRODUCTION) {
            while (true) {
                try {
                    await fetch(AI_SERVER_HOST);
                    break;
                } catch (e) {
                    logger.debug(
                        `Couldn't get a response from ${AI_SERVER_HOST}. Retrying in ${RETRY_INTERVAL.asSeconds()} seconds...`
                    );
                    await new Promise((r) => setTimeout(r, RETRY_INTERVAL.asMilliseconds()));
                }
            }
        }

        app.use(express.static("public"));

        app.listen(PORT, HOST, () => {
          logger.info("Web server is listening http://%s:%s", HOST, PORT);
        });
    })();
}

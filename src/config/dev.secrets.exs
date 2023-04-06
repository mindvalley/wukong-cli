use Mix.Config

# Configure your database.
config :academy, Academy.Repo,
  adapter: Ecto.Adapters.Postgres,
  username: "postgres",
  password: "",
  database: "academy_core_dev",
  hostname: "localhosts",
  pool_size: 10

# Play store.
config :academy, :play_store,
  public_key:
    "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEApcjVcosS6VWUPwDBo6640p9djX6PnLicsKvRF11UQlPL2oKClAvHmY10FqAFVa7FbFcMqJloq3H/bQPHfNnZxelYruiFAlG1C51sDEo244AvMn2Frto9g86uGTAGETywKAPkPffiMltgezmpNwJy1Q4hfLrmxZnmUPcdg98+pElnsdR7ev3uspFIYB12kPo0yDuJsEJ8AhmVJGocE2jp6C3rnrcbyosYxcknoJyFXP/MDE7MP1fhWDLk3Mw3Vy8YilTkHuxPeR6id7/00Rprgs57jkLSgBvkCG+qdLpJhYId8QsonrW5VI34JHQ4aYahwRFbznOzPkkwK4K9dQP5gQIDAQAB"

# Get this service key to ensure that asset uploading is working.
config :academy, :fineuploader,
  key: "<KEY>",
  secret: "<SECRET>",
  bucket: "<BUCKET>",
  s3endpoint: "s3-accelerate.amazonaws.com",
  upload_dir: "tmp/",
  notify_url: ""

# ???
config :receipt_verifier,
  shared_secret: "<SECRET>"

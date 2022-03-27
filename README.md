# Cloud Storage Job Runner - `csjr`

Download files from a storage bucket in the cloud, run a job on each of them, and upload the results back to the cloud.

## Use Case Examples

### Running Inference on Images

Imagine you've trained an image segmentation model, and you'd like to feed new images into it as input data. This model isn't deployed in production anywhere just yet; for now, you've written a script that runs images through this model and saves the resulting annotated images to disk. Meanwhile, when new images are collected and are ready for segmentation, someone else on your team uploads them to a storage bucket in the cloud that you have access to. To annotate those images, you have to download them, run your script, and upload the results to that bucket — all manually. `csjr` **can automate this entire process.**

## FAQ

### Why save results to disk before uploading them back to the cloud?

`csjr` is meant to wrap around what you've already built. You shouldn't have to write new code or change how you're doing things in order for `csjr` to help you automate something.

You may also still want to be able to run a job manually from time to time, whether that be for quickly testing something, a one-off job, or whatever else. With `csjr`, you don't have to maintain code or infrastructure for an automated version _and_ a manual version of your process — `csjr` works with what you already have out of the box.

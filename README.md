# Cloud Storage Job Runner - `csjr`

Download files from a storage bucket in the cloud, run a job on each of them, and upload the results back to the cloud.

## Usage

1. Write a config YAML file with jobs you want to perform
   - See [example-config.yaml](example-config.yaml)
1. Run `csjr`, pointing to that config file
   - `$ csjr -c path/to/config/file`
     - Runs all the jobs defined in the config
   - `$ csjr -c path/to/config/file job-name`
     - Only runs the specified job

### Specifying Input and Output Directories

When `csjr` downloads files from the cloud, it downloads an entire folder specified by the job's `path-to-remote-inputs`. It saves that folder to disk at the job's specified `path-to-local-inputs` directory.

When `csjr` uploads output files back to the cloud, it looks for the folder on disk specified by the job's `path-to-local-outputs`. It uploads that entire folder to the cloud at the job's specified `path-to-remote-outputs` directory.

### Writing Steps

The steps you write for a job should be the same steps that you run on the command line to perform the job manually. Each of a job's steps will be run in a sub-shell. In other words, when `csjr` is running a job, it shells out to each of the steps defined in the config for that job.

When you write `[path-to-local-inputs]` or `[path-to-local-outputs]` in one of your steps, `csjr` will substitute it with the `path-to-local-inputs` or `path-to-local-outputs` directory specified in that job's config, respectfully.

## Use Case Examples

### Running Inference on Images

Imagine you've trained an image segmentation model, and you'd like to feed new images into it as input data. This model isn't deployed in production anywhere just yet; for now, you've written a script that runs images through this model and saves the resulting annotated images to disk. Meanwhile, when new images are collected and are ready for segmentation, someone else on your team uploads them to a storage bucket in the cloud that you have access to. To annotate those images, you have to download them, run your script, and upload the results to that bucket — all manually. `csjr` **can automate this entire process.**

## FAQ

### Why save results to disk before uploading them back to the cloud?

`csjr` is meant to wrap around what you've already built. You shouldn't have to write new code or change how you're doing things in order for `csjr` to help you automate something.

You may also still want to be able to run a job manually from time to time, whether that be for quickly testing something, a one-off job, or whatever else. With `csjr`, you don't have to maintain code or infrastructure for an automated version _and_ a manual version of your process — `csjr` works with what you already have out of the box.

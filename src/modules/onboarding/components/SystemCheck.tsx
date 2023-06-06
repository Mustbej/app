import { Tooltip } from "react-tooltip";
import Logo from "assets/images/brand-logo.svg";
import arrow from "assets/images/arrow.svg";
import computerLogo from "assets/images/computer.svg";
import appleLogo from "assets/images/apple.svg";
import { SystemCheckProps } from "../types";
import Dependency from "./Dependency";
import PrimaryButton from "shared/components/PrimaryButton";
import useSystemStats from "shared/hooks/useSystemStats";
import Spinner from "shared/components/Spinner";

const SystemCheck = ({
  handleCheckIsDockerRunning,
  isDockerRunning,
  isServerRunning,
  back,
  next,
}: SystemCheckProps) => {
  const { data: response, isLoading } = useSystemStats();

  const memoryLimit = response?.data?.memory_limit ? `${response?.data?.memory_limit}GiB` : "";

  return (
    <section className="system-check flex flex-wrap bg-lines relative">
      <div className="bg-tulip w-full py-2 sm:max-h-[56px] text-center max-sm:px-1 z-[11]">
        <p className="text-brightgray text-sm">
          Don't have requirements? Try the &nbsp;
          <a href="https://app.prem.ninja/" target="_blank" className="underline decoration">
            Prem App
          </a>
          &nbsp; demo.
        </p>
      </div>
      <div className="welcome-logo z-[1] w-full">
        <img src={Logo} alt="logo" />
        <button onClick={back}>
          <img src={arrow} alt="arrow" />
        </button>
      </div>
      <div className="relative z-[1] mx-auto max-md:w-full system-check__container">
        <div className="mx-auto md:max-w-[600px] max-md:px-6 w-full rounded-xl">
          <div className="">
            <div className="md:max-w-[450px] mx-auto relative z-10">
              <img
                className="mx-auto w-[85px] h-[60px] my-[30px]"
                src={computerLogo}
                alt="computerLogo"
              />
              <div className="text-center mx-auto mt-5">
                <h1 className="text-xl dark:text-brightgray">System Check</h1>
                <p className="dark:text-brightgray text-sm font-proximaNova-regular my-4">
                  For Prem App to run smoothly you need:
                </p>
              </div>
              <div className="system-check__requirements">
                <h3 className="text-base dark:text-white">Requirements:</h3>
                <Dependency
                  isRunning={isDockerRunning}
                  status={isDockerRunning ? "Found" : "Not Found"}
                  name="Docker"
                  id={"docker"}
                  tooltip={<div>Prem App requires Docker installed and running.</div>}
                />
                <Dependency
                  isRunning={isServerRunning}
                  status={isServerRunning ? "Running" : "Not Running"}
                  name="Daemon"
                  id={"daemon"}
                  tooltip={
                    <div>
                      Prem App requires Prem Daemon to be up and running. <br /> The Daemon is
                      automatically started when docker is running.
                    </div>
                  }
                />

                <Dependency
                  isLoading={isLoading}
                  isRunning={false}
                  status={memoryLimit}
                  name="Memory"
                  id={"memory"}
                  tooltip={
                    <div>
                      Prem App you requires at least <br /> {memoryLimit} of RAM allocated for
                      Docker Engine.
                    </div>
                  }
                />
              </div>
              <div className="text-center mt-1 mb-16">
                <button
                  className="text-white text-sm opacity-70"
                  onClick={() => handleCheckIsDockerRunning()}
                >
                  Check Again
                </button>
              </div>
            </div>
            <div className="grid sm:grid-cols-2 gap-2 items-center flex-wrap">
              <a
                className="border border-white text-white !rounded-[6px] font-proximaNova-regular py-3 px-[22px] flex justify-center"
                href="https://www.docker.com/products/docker-desktop/"
                target="_blank"
              >
                <img className="mr-3" src={appleLogo} alt="appleLogo" />
                Download Docker Desktop
              </a>
              <PrimaryButton
                disabled={!(isDockerRunning && isServerRunning)}
                className="!px-9 sm:ml-4 !h-[50px] cursor-pointer"
                onClick={next}
              >
                Next
              </PrimaryButton>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

export default SystemCheck;

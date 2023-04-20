import React, {useState} from 'react';
import Image01           from 'website/sources/template/images/content-image-01.jpg';
import Image02           from 'website/sources/template/images/content-image-02.jpg';
import {Modal}           from 'website/sources/template/utils/Modal';

export function GuidesContent() {
  const [videoModalOpen, setVideoModalOpen] = useState(false);

  return (
    <div>
      <header className={`mb-6`}>
        <h1 className={`h2 text-slate-800 mb-4 dark:text-slate-200`}>Marketing API Quick Start</h1>
        <p className={`text-lg text-slate-600 dark:text-slate-400`}>
          This guide will give you everything you need to start using the Docs Marketing API to manage audiences, control automation workflows, sync
          email activity with your database, and more.
        </p>
      </header>
      <div className={`text-slate-600 dark:text-slate-400 space-y-6`}>
        {/* Article section */}
        <div className={`space-y-6`}>
          <div className={`space-y-4`}>
            {/* HTML Bookmark */}
            {/* The data-scrollspy-target attribute makes the scrollspy work */}
            <h2 id={`at-a-glance`} data-scrollspy-target className={`h4 text-slate-800 scroll-mt-24 dark:text-slate-200`}>
              At a glance
            </h2>
            <p>
              We'll walk through generating your API key, installing the client library for your preferred language, and making your first API call—a
              simple request to the Ping endpoint.
            </p>
          </div>
        </div>

        {/* Article section */}
        <div className={`space-y-6`}>
          <div className={`space-y-4`}>
            {/* HTML Bookmark */}
            {/* The data-scrollspy-target attribute makes the scrollspy work */}
            <h2 id={`generate-api`} data-scrollspy-target className={`h4 text-slate-800 scroll-mt-24 dark:text-slate-200`}>
              Generate your API key
            </h2>
            <p>
              The simplest way to authenticate a request to the Marketing API is using an{` `}
              <a className={`text-blue-600 font-medium hover:underline`} href={`#0`}>
                API key
              </a>
              .
            </p>
            <ul className={`list-inside space-y-2 pl-4`}>
              <li>
                <span className={`text-slate-400 dark:text-slate-500 pr-2`}>↳</span> Navigate to the API Keys section of your Docs account.
              </li>
              <li>
                <span className={`text-slate-400 dark:text-slate-500 pr-2`}>↳</span> If you already have an API key listed to use for your application,
                simply copy it.
              </li>
            </ul>
            <p>Let's see an example:</p>

            {/* Image vith modal video */}
            <div>
              <div className={`relative inline-flex justify-center items-center my-2`}>
                <img className={`rounded`} src={Image01} width={`680`} height={`382`} alt={`Content image 01`} />
                <button className={`absolute group`} onClick={e => {
                  e.preventDefault(); e.stopPropagation(); setVideoModalOpen(true);
                }} aria-controls={`video-modal`}>
                  <svg className={`w-16 h-16 fill-current sm:w-20 sm:h-20 group`} viewBox={`0 0 88 88`} xmlns={`http://www.w3.org/2000/svg`}>
                    <circle className={`text-white opacity-80 group-hover:opacity-100 transition duration-150 ease-in-out`} cx={`44`} cy={`44`} r={`44`} />
                    <path
                      className={`text-blue-600`}
                      d={`M52 44a.999.999 0 00-.427-.82l-10-7A1 1 0 0040 37V51a.999.999 0 001.573.82l10-7A.995.995 0 0052 44V44c0 .001 0 .001 0 0z`}
                    />
                  </svg>
                </button>
              </div>
              <Modal id={`video-modal`} ariaLabel={`modal-headline`} show={videoModalOpen} handleClose={() => setVideoModalOpen(false)}>
                <div className={`relative pb-9/16`}>
                  <iframe className={`w-full aspect-video`} src={`https://player.vimeo.com/video/174002812`} title={`Video`} allowFullScreen></iframe>
                </div>
              </Modal>
            </div>

            <p>
              If you're creating integrations that require access to Mailchimp on behalf of other Docs users, you'll want to{` `}
              <a className={`text-blue-600 font-medium hover:underline`} href={`#0`}>
                set up authentication via Oauth 2
              </a>{` `}
              instead.
            </p>
          </div>
        </div>

        {/* Article section */}
        <div className={`space-y-6`}>
          <div className={`space-y-4`}>
            {/* HTML Bookmark */}
            {/* The data-scrollspy-target attribute makes the scrollspy work */}
            <h2 id={`install`} data-scrollspy-target className={`h4 text-slate-800 scroll-mt-24 dark:text-slate-200`}>
              Install the client library for your language
            </h2>
            <p>
              You can make calls to the Marketing API with whichever method you usually use to make HTTP requests, but Docs offers client libraries
              that make interacting with the API even simpler.
            </p>
            <p>To install the client library for your preferred language:</p>
            <div className={`my-2`}>
              <img className={`rounded`} src={Image02} width={`680`} height={`382`} alt={`Content image 02`} />
            </div>
          </div>
        </div>

        {/* Article section */}
        <div className={`space-y-6`}>
          <div className={`space-y-4`}>
            {/* HTML Bookmark */}
            {/* The data-scrollspy-target attribute makes the scrollspy work */}
            <h2 id={`api-call`} data-scrollspy-target className={`h4 text-slate-800 scroll-mt-24 dark:text-slate-200`}>
              Make your first API call
            </h2>
            <p>
              To test that you have everything set up correctly, we'll make a simple request to the Ping endpoint. Hitting this endpoint acts as a
              health check on the Docs API service; it won't affect your account in any way.
            </p>
            <p>
              To find the value for the server parameter used in{` `}
              <span className={`font-medium text-slate-800 underline dark:text-slate-200`}>docs.setConfig</span>, log into your Docs account and look at
              the URL in your browser. You'll see something like{` `}
              <span className={`font-medium text-slate-800 underline dark:text-slate-200`}>https://us19.admin.docs.com/</span>; the us19 part is the
              server prefix. Note that your specific value may be different.
            </p>
            <pre className={`overflow-x-auto text-sm text-slate-400 bg-slate-800 border border-slate-700 p-4 rounded`}>
              <code className={`font-pt-mono`}>
                # install jq: https://docs.github.io/jq/download/{`\n\n`}
                <span className={`text-teal-500`}>dc</span>
                <span className={`text-rose-400`}>=</span>
                <span className={`text-purple-500`}>"YOUR_DC"</span>
                {`\n\n`}
                <span className={`text-teal-500`}>apikey</span>
                <span className={`text-rose-400`}>=</span>
                <span className={`text-purple-500`}>"YOUR_API_KEY"</span>
                {`\n\n`}
                <span className={`text-teal-500`}>curl</span> -sS \{`\n\n`}
                {`\t`}
                <span className={`text-purple-500`}>"https://</span>
                <span className={`text-teal-500`}>$&#123;dc&#125;</span>
                <span className={`text-purple-500`}>.api.docs.com/3.0/ping"</span> \{`\n\n`}
                {`\t`}--user <span className={`text-purple-500`}>"anystring:</span>
                <span className={`text-teal-500`}>$&#123;apikey&#125;</span>
                <span className={`text-purple-500`}>"</span> <span className={`text-rose-400`}>|</span> jq -r
              </code>
            </pre>
            <p>
              If everything was set up correctly and the request to{` `}
              <span className={`font-medium text-slate-800 underline dark:text-slate-200`}>ping</span> was a success, the response should look like the
              following:
            </p>
          </div>
        </div>

        {/* Article section */}
        <div className={`space-y-6`}>
          <div className={`space-y-4`}>
            {/* HTML Bookmark */}
            {/* The data-scrollspy-target attribute makes the scrollspy work */}
            <h2 id={`config-status`} data-scrollspy-target className={`h4 text-slate-800 scroll-mt-24 dark:text-slate-200`}>
              Configuration status
            </h2>
            <p>
              There are a number of configuration status which are explained in detail in this article. Below is an index of all configuration status:
            </p>
            {/* Table */}
            <div className={`overflow-x-auto`}>
              <table className={`table-auto w-full my-2 border-b border-slate-200 dark:border-slate-800`}>
                {/* Table header */}
                <thead>
                  <tr className={`text-left text-slate-800 dark:text-slate-200`}>
                    <th className={`font-medium px-2 first:pl-0 last:pr-0 py-3`}>Status</th>
                    <th className={`font-medium px-2 first:pl-0 last:pr-0 py-3`}>Description</th>
                  </tr>
                </thead>
                {/* Table body */}
                <tbody>
                  {/* Row */}
                  <tr className={`border-t border-slate-200 dark:border-slate-800`}>
                    <td className={`px-2 first:pl-0 last:pr-0 py-3`}>
                      <div className={`text-sm inline-flex font-medium bg-teal-500 bg-opacity-25 text-teal-600 rounded text-center px-1`}>
                        Subscribed
                      </div>
                    </td>
                    <td className={`px-2 first:pl-0 last:pr-0 py-3`}>The contact is subscribed to the list and can receive campaigns.</td>
                  </tr>
                  {/* Row */}
                  <tr className={`border-t border-slate-200 dark:border-slate-800`}>
                    <td className={`px-2 first:pl-0 last:pr-0 py-3`}>
                      <div className={`text-sm inline-flex font-medium bg-rose-500 bg-opacity-25 text-rose-500 rounded text-center px-1`}>
                        Unsubscribed
                      </div>
                    </td>
                    <td className={`px-2 first:pl-0 last:pr-0 py-3`}>The contact is no longer subscribed to the list.</td>
                  </tr>
                  {/* Row */}
                  <tr className={`border-t border-slate-200 dark:border-slate-800`}>
                    <td className={`px-2 first:pl-0 last:pr-0 py-3`}>
                      <div className={`text-sm inline-flex font-medium bg-purple-500 bg-opacity-25 text-purple-600 rounded text-center px-1`}>
                        Cleaned
                      </div>
                    </td>
                    <td className={`px-2 first:pl-0 last:pr-0 py-3`}>The contact bounced and was removed from the list.</td>
                  </tr>
                  {/* Row */}
                  <tr className={`border-t border-slate-200 dark:border-slate-800`}>
                    <td className={`px-2 first:pl-0 last:pr-0 py-3`}>
                      <div className={`text-sm inline-flex font-medium bg-blue-500 bg-opacity-25 text-blue-500 rounded text-center px-1`}>Pending</div>
                    </td>
                    <td className={`px-2 first:pl-0 last:pr-0 py-3`}>The contact has not yet confirmed their subscription.</td>
                  </tr>
                </tbody>
              </table>
            </div>
            <p>
              To test that you have everything set up correctly, we'll make a simple request to the Ping endpoint. Hitting this endpoint acts as a
              health check on the Docs API service; it won't affect your account in any way.
            </p>
            <p>
              HTTP requests are the backbone of the internet. Without them, we wouldn't be able to communicate with web servers and load the pages we
              see in our browser.
            </p>
            <p>This library is available on most Unix-like systems, and can be used to make HTTP requests to any HTTP server.</p>
          </div>
        </div>
      </div>
    </div>
  );
}

import React       from 'react';
import {Accordion} from 'website/sources/template/utils/Accordion';

export function HelpContent() {
  return (
    <div>
      <header className={`mb-8`}>
        <h1 className={`text-4xl font-[650] text-slate-800 mb-4 dark:text-slate-200`}>How can we help?</h1>
        <p className={`text-lg text-slate-600 dark:text-slate-400`}>
          Everything you need to know right here at your fingertips. Ask questions, browse around for answers, or submit your feature requests.
        </p>
      </header>
      <div className={`text-slate-600 dark:text-slate-400 space-y-8`}>
        {/* Faqs section */}
        <div className={`space-y-8 mb-12 md:mb-16`}>
          <section className={`space-y-4`}>
            {/* HTML Bookmark */}
            {/* The data-scrollspy-target attribute makes the scrollspy work */}
            <h2 id={`overview`} data-scrollspy-target className={`text-2xl font-[650] text-slate-800 scroll-mt-24 dark:text-slate-200`}>
              Overview
            </h2>
            <ul className={`space-y-3`}>
              <Accordion title={`Does Docs support other data sources, like Excel or Airtable?`} open={true}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
              <Accordion title={`Can I collaborate on campaigns with others?`}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
              <Accordion title={`Will you help me build my campaign?`}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
              <Accordion title={`How can I become a partner?`}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
            </ul>
          </section>

          <section className={`space-y-4`}>
            {/* HTML Bookmark */}
            {/* The data-scrollspy-target attribute makes the scrollspy work */}
            <h2 id={`support`} data-scrollspy-target className={`text-2xl font-[650] text-slate-800 scroll-mt-24 dark:text-slate-200`}>
              Support
            </h2>
            <ul className={`space-y-3`}>
              <Accordion title={`How can I talk to someone at Docs?`} open={true}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
              <Accordion title={`Why should I connect my clients?`}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
              <Accordion title={`Can I collaborate on campaigns with others?`}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
              <Accordion title={`Is Glide Docs and CCPA compliant?`}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
            </ul>
          </section>

          <section className={`space-y-4`}>
            {/* HTML Bookmark */}
            {/* The data-scrollspy-target attribute makes the scrollspy work */}
            <h2 id={`security`} data-scrollspy-target className={`text-2xl font-[650] text-slate-800 scroll-mt-24 dark:text-slate-200`}>
              Security &amp; Privacy
            </h2>
            <ul className={`space-y-3`}>
              <Accordion title={`What are Private Users of Private Pro campaigns?`} open={true}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
              <Accordion title={`Does Docs sell or share the content of my campaigns with third parties?`}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
              <Accordion title={`What access does Docs have to my campaigns?`}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
            </ul>
          </section>

          <section className={`space-y-4`}>
            {/* HTML Bookmark */}
            {/* The data-scrollspy-target attribute makes the scrollspy work */}
            <h2 id={`troubleshooting`} data-scrollspy-target className={`text-2xl font-[650] text-slate-800 scroll-mt-24 dark:text-slate-200`}>
              Troubleshooting
            </h2>
            <ul className={`space-y-3`}>
              <Accordion title={`An error occurred deleting my campaign`} open={true}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
              <Accordion title={`Something went wrong adding the webhook`}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
              <Accordion title={`Something went wrong updating your payment information`}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
              <Accordion title={`Something went wrong trying to delete that user's data`}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
              <Accordion title={`Something went wrong replacing the campaign`}>
                <p>
                  There are many variations of passages available, but the majority have suffered alteration in some form, by injected humour, or
                  randomised words which don't look. If you are going to use a passage of Lorem Ipsum.
                </p>
              </Accordion>
            </ul>
          </section>
        </div>
      </div>
    </div>
  );
}

#include <stdio.h>

#ifdef LIBGIT_THREADS_INIT
# include <git2/threads.h>
#endif
#ifdef LIBGIT_GLOBAL_INIT
# include <git2/global.h>
#endif

#include <git2/buffer.h>
#include <git2/branch.h>
#include <git2/errors.h>
#include <git2/repository.h>

void list_branches(git_repository *repo) {
	int
		error;
	git_branch_iterator
		*branch_iterator = NULL;
	git_reference
		*reference = NULL;
	git_branch_t
		branch_type;
	const char
		*branch_name;

	error = git_branch_iterator_new(&branch_iterator, repo, GIT_BRANCH_ALL);
	if (0 != error) {
		const git_error *e = giterr_last();
		fprintf(stderr, "Got error initializing branch iterator: %s\n", e->message);
		return;
	}

	while (0 == (error = git_branch_next(&reference, &branch_type, branch_iterator))) {
		error = git_branch_name(&branch_name, reference);
		if (0 != error) {
			const git_error *e = giterr_last();
			fprintf(stderr, "Got error iterating over branches: %s\n", e->message);
		}

		fprintf(stdout, "- %s\n", branch_name);
	}
	if (GIT_ITEROVER != error) {
		const git_error *e = giterr_last();
		fprintf(stderr, "Got error iterating over branches: %s\n", e->message);
	}


	if (NULL != branch_iterator) {
		git_branch_iterator_free(branch_iterator);
	}
}

int main(int argc __attribute__((unused)), char** argv __attribute__((unused))) {
	int
		error;
	git_repository
		*repo = NULL;

	if (argc < 2) {
		fprintf(stderr, "No path given\n");
		return 1;
	}

#ifdef LIBGIT_GLOBAL_INIT
	error = git_libgit2_init();
	if (1 < error) {
		const git_error *e = giterr_last();
		fprintf(stderr, "Got error initializing libgit2: %s\n", e->message);
	}
#endif
#ifdef LIBGIT_THREADS_INIT
	error = git_threads_init();
	if (1 < error) {
		const git_error *e = giterr_last();
		fprintf(stderr, "Got error initializing libgit2 threads: %s\n", e->message);
	}
#endif

	error = git_repository_open_ext(&repo, argv[1], 0, NULL);
	if (0 > error) {
		if (GIT_ENOTFOUND == error) {
			fprintf(stderr, "Could not find repo\n");
		}
		else {
			const git_error *e = giterr_last();
			fprintf(stderr, "Got an error opening repository: %s\n", e->message);
		}
	}
	else {
		//fprintf(stderr, "We are in a git repo\n");
	}

	list_branches(repo);

	if (NULL != repo) {
		git_repository_free(repo);
	}

#ifdef LIBGIT_THREADS_INIT
	git_threads_shutdown();
#endif
#ifdef LIBGIT_GLOBAL_INIT
	while(0 < git_libgit2_shutdown()) {};
#endif
	return 0;
}

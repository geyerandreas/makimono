import pymakimono


def test_generate_content_smoke():
    result = pymakimono.generate_content(
        "### Latest Changes\n",
        "* New & first PR Feature",
    )

    assert result == "### Latest Changes\n\n* New & first PR Feature\n"

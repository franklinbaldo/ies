"""Ajustes à mão nos testes, cada um com o motivo."""

import glob
import os
import re

from ajustes_base import ERRO, ajuste, trocar, trocar_re


# --- testes: a variante depende de qual arquivo falhou -----------------------


@ajuste("src/transporte/tests.rs", "a URL por link falha no arquivo da URL do Controle")
def _(t):
    return trocar(
        t,
        """            &raiz.join("origem-nao-existe"),
        ),
        Err(crate::arquivo_privado::ErroArquivoPrivado::LinkSimbolico)""",
        f"""            &raiz.join("origem-nao-existe"),
        ),
        Err(FalhaTransporte::UrlDoControleIlegivel(
            {ERRO}::LinkSimbolico
        ))""",
    )


@ajuste(
    "src/sessao_login/tests_snapshot.rs",
    "cada arquivo da sessão tem a sua variante em FalhaTransporte",
)
def _(t):
    t = trocar(
        t,
        """    for nome in [&geracao, &arquivos.url_controle, &arquivos.origem] {""",
        """    let link = ErroArquivoPrivado::LinkSimbolico;
    for (nome, esperada) in [
        (&geracao, FalhaTransporte::MarcadorDaSessaoIlegivel(link)),
        (&arquivos.url_controle, FalhaTransporte::UrlDoControleIlegivel(link)),
        (&arquivos.origem, FalhaTransporte::OrigemIlegivel(link)),
    ] {""",
    )
    t = trocar(
        t,
        """            Some(crate::arquivo_privado::ErroArquivoPrivado::LinkSimbolico)""",
        """            Some(esperada)""",
    )
    t = trocar(
        t,
        """        Err(crate::arquivo_privado::ErroArquivoPrivado::LinkSimbolico)""",
        """        Err(FalhaTransporte::CookieDaSessaoIlegivel(
            ErroArquivoPrivado::LinkSimbolico
        ))""",
    )
    return trocar(
        t,
        "use crate::transporte::FalhaTransporte;\n",
        "use crate::arquivo_privado::ErroArquivoPrivado;\nuse crate::transporte::FalhaTransporte;\n",
    )


@ajuste("src/formulario_web/tests_envio.rs", "a permissão aberta é do arquivo do cookie")
def _(t):
    return trocar_re(
        t,
        r"ErroSubmissaoFormulario::Transporte\(\s*crate::arquivo_privado::ErroArquivoPrivado::PermissoesAbertas\s*\)",
        f"ErroSubmissaoFormulario::Transporte(FalhaTransporte::CookieDaSessaoIlegivel(\n"
        f"            {ERRO}::PermissoesAbertas\n        ))",
    )


@ajuste(
    "src/unidade_trocar_cli/tests.rs",
    "aplicar_troca recebe o cookie já conferido pela troca: o caso do cookie ausente "
    "deixou de ser representável e sai; a falha de gravação é o ErroLogin exato",
)
def _(t):
    t = trocar_re(
        t,
        r"(?s)\n    let cookie_ausente = CookieDaSessao::ler\(&dir\.join\(\"cookie-inexistente\"\)\);\n"
        r"    assert!\(matches!\(\n.*?\n    \)\);\n",
        "\n",
    )
    t = trocar(t, "&CookieDaSessao::sintetico(COOKIE),", "COOKIE,", 2)
    t = trocar(
        t,
        "        controle_novo: Some(lido(url_limpa(ID_B))),\n    };",
        "        controle_novo: Some(lido(url_limpa(ID_B))),\n"
        "        cookie: crate::transporte::sessao::CookieLido::sintetico(COOKIE),\n    };",
    )
    return trocar(
        t,
        "Err(FalhaAplicarTroca::Gravacao(_))",
        "Err(crate::sessao_login::ErroLogin::DestinoDaSessaoNaoEhArquivoRegular(\n"
        "            crate::configuracao::ArquivoPrivado::UrlControle\n        ))",
    )


# --- testes da CLI: falha_por precisa do módulo de apoio --------------------


@ajuste(
    "tests/cli_unidade_trocar.rs",
    "o nome esperado vem de uma variável, fora do alcance da regra 40",
)
def _(t):
    t = trocar(
        t,
        """        let erro = if confirmar {
            "ArquivoDeLockDoLoginInacessivel"
        } else {
            "ArquivoPrivadoAusenteOuInacessivel"
        };
        falha(&saida, erro);""",
        """        if confirmar {
            falha(&saida, "ArquivoDeLockDoLoginInacessivel");
        } else {
            falha_por(&saida, "UrlDoControleIlegivel", "Ausente");
        }""",
    )
    return t


@ajuste(
    "tests/cli_unidade_snapshot_offline.rs",
    "a causa antiga (um nome) vira duas: o arquivo (cookie) e o fato (ausente)",
)
def _(t):
    return trocar(
        t,
        """    falha_por(
        &atualizar,
        "LupaSemListaIntegral",
        "ArquivoPrivadoAusenteOuInacessivel",
    );""",
        """    falha_por(&atualizar, "LupaSemListaIntegral", "CookieDaSessaoIlegivel");
    falha_por(&atualizar, "LupaSemListaIntegral", "Ausente");""",
    )


def modulos_de_apoio():
    """Cada binário de teste que passou a chamar falha_por (regra 40) ganha o
    módulo support/falha_por.rs, que exige o de falha.rs como `falha_cli`;
    o `use` de `falha` sai de quem não o chama mais."""
    for raiz in sorted(glob.glob("tests/*.rs")):
        pasta = raiz[: -len(".rs")]
        textos = [
            open(f, encoding="utf-8").read()
            for f in glob.glob(os.path.join(pasta, "**", "*.rs"), recursive=True)
        ]
        t = open(raiz, encoding="utf-8").read()
        todos = "\n".join([t, *textos])
        if "falha_por(" not in todos or "mod falha_por_cli;" in t:
            continue
        t = trocar(
            t,
            '#[path = "support/falha.rs"]\nmod support;\nuse support::falha;\n',
            '#[path = "support/falha.rs"]\nmod falha_cli;\nuse falha_cli::falha;\n',
            t.count("mod support;\nuse support::falha;"),
        )
        apoio = (
            '#[path = "support/falha_por.rs"]\nmod falha_por_cli;\nuse falha_por_cli::falha_por;\n'
        )
        if re.search(r"(?<![_A-Za-z])falha\(", todos):
            t = trocar(t, "use falha_cli::falha;\n", "use falha_cli::falha;\n" + apoio)
        else:
            t = trocar(t, "use falha_cli::falha;\n", apoio)
        with open(raiz, "w", encoding="utf-8") as f:
            f.write(t)

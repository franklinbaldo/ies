"""Ajustes à mão no código de produção, cada um com o motivo."""

from ajustes_base import ERRO, ajuste, trocar, trocar_re


# --- arquivo_privado: a leitura entra no módulo da gravação -----------------


@ajuste(
    "src/arquivo_privado.rs",
    "declarar o submódulo leitura (modelos/leitura.rs) e reexportar ler e o erro",
)
def _(t):
    t = trocar(
        t,
        "//! Gravação privada única (#211): o par de escrita de\n"
        "//! `transporte::arquivo_privado`, que é a leitura. Cria",
        "//! Gravação privada única (#211): o par de escrita de [`ler`], que é\n"
        "//! a leitura. Cria",
    )
    return trocar(
        t,
        "pub(crate) use trava::{ErroDaTrava, TravaDeArquivo, travar_com_espera};\nmod trava;\n",
        "pub(crate) use leitura::{ErroArquivoPrivado, ler};\n"
        "pub(crate) use trava::{ErroDaTrava, TravaDeArquivo, travar_com_espera};\n"
        "mod leitura;\nmod trava;\n",
    )


# --- transporte: as variantes do arquivo saem; o transporte embrulha uma vez -


@ajuste(
    "src/transporte/falha.rs",
    "as 9 variantes do arquivo saem do enum e dos 3 match; entram 4, uma por arquivo que o transporte lê",
)
def _(t):
    t = trocar(
        t,
        """    /// O arquivo não existe ou não pode ser consultado (o motivo do sistema).
    ArquivoPrivadoAusenteOuInacessivel(std::io::ErrorKind),
    #[cfg(unix)]
    ArquivoPrivadoComPermissoesInseguras,
    CaminhoDeArquivoNaoEhArquivo,
    ArquivoPrivadoLinkSimbolico,
    ArquivoPrivadoVazio,
    ArquivoPrivadoAcimaDoLimite,
    /// O arquivo não pôde ser aberto ou lido (o motivo do sistema).
    ArquivoPrivadoNaoPodeSerLido(std::io::ErrorKind),
    /// O conteúdo do arquivo não é UTF-8 (só a posição do erro, sem o texto).
    ArquivoPrivadoNaoUtf8(#[source] std::str::Utf8Error),
""",
        """    /// O arquivo da URL assinada do Controle não foi lido.
    UrlDoControleIlegivel(#[source] ErroArquivoPrivado),
    /// O arquivo da origem autorizada não foi lido.
    OrigemIlegivel(#[source] ErroArquivoPrivado),
    /// O arquivo do cookie da sessão não foi lido.
    CookieDaSessaoIlegivel(#[source] ErroArquivoPrivado),
    /// O marcador `.geracao` existe e não foi lido.
    MarcadorDaSessaoIlegivel(#[source] ErroArquivoPrivado),
""",
    )
    t = trocar_re(
        t,
        r"\n            #\[cfg\(unix\)\]\n            Self::ArquivoPrivadoComPermissoesInseguras => [^\n]*,",
        "",
        3,
    )
    t = trocar_re(
        t,
        r"\n            \| Self::(ArquivoPrivado[A-Za-z0-9]*|CaminhoDeArquivoNaoEhArquivo)(\(_\))?(?=\n)",
        "",
        21,
    )
    t = trocar(
        t,
        "            | Self::SessaoTrocadaDuranteALeitura\n",
        "            | Self::UrlDoControleIlegivel(_)\n"
        "            | Self::OrigemIlegivel(_)\n"
        "            | Self::CookieDaSessaoIlegivel(_)\n"
        "            | Self::MarcadorDaSessaoIlegivel(_)\n"
        "            | Self::SessaoTrocadaDuranteALeitura\n",
        3,
    )
    return trocar(
        t,
        "//! (docs/erros-tipados.md). Contrato: contracts/cli.md.\n",
        "//! (docs/erros-tipados.md). Contrato: contracts/cli.md.\n"
        "use crate::arquivo_privado::ErroArquivoPrivado;\n",
    )


@ajuste("src/transporte.rs", "std::fs só servia à leitura que saiu (os testes passam a importá-lo)")
def _(t):
    return trocar(t, "use std::fs;\n", "")


@ajuste("src/transporte/tests.rs", "std::fs vinha de transporte.rs pelo glob")
def _(t):
    return trocar(t, "use super::*;\n", "use super::*;\nuse std::fs;\n")


# --- login: CredenciaisIlegiveis, OrigemIlegivel e SessaoGravadaInvalida -----


@ajuste(
    "src/sessao_login/credenciais.rs",
    "CredenciaisIlegiveis (uma variante só) sai; o erro do arquivo vai direto a ErroLogin",
)
def _(t):
    t = trocar(
        t, ".map_err(CredenciaisIlegiveis::Arquivo)?", ".map_err(ErroLogin::ArquivoDeCredenciais)?"
    )
    return trocar(
        t,
        """
/// O arquivo privado de credenciais ausente, inseguro ou ilegível. Tipo próprio da etapa; o nome
/// público é o motivo do transporte (`transparente`).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum CredenciaisIlegiveis {
    Arquivo(#[from] crate::transporte::FalhaTransporte),
}
""",
        "",
    )


@ajuste(
    "src/sessao_login/erro.rs",
    "as variantes de arquivo levam o ErroArquivoPrivado sem envelope; a origem fora do formato vira fato próprio",
)
def _(t):
    t = trocar(
        t,
        "use crate::transporte::FalhaTransporte;\n",
        "use crate::arquivo_privado::ErroArquivoPrivado;\nuse crate::transporte::FalhaTransporte;\n",
    )
    t = trocar(
        t,
        """    /// Origem da instalação ilegível ou fora do formato `Instalacao`.
    Origem(#[from] super::fluxo::OrigemIlegivel),
""",
        """    /// O arquivo da origem da instalação não foi lido.
    OrigemIlegivel(#[source] ErroArquivoPrivado),
    /// A origem lida não é uma instalação (`Instalacao`).
    OrigemNaoAutorizada,
""",
    )
    t = trocar(
        t,
        "    ArquivoDeCredenciais(#[from] super::credenciais::CredenciaisIlegiveis),\n",
        "    /// O arquivo de credenciais não foi lido.\n"
        "    ArquivoDeCredenciais(#[source] ErroArquivoPrivado),\n",
    )
    t = trocar(
        t,
        """    /// URL ou cookie gravados que o transporte recusa antes da rede.
    ArquivoDaSessao(#[from] super::fluxo::SessaoGravadaInvalida),
""",
        """    /// O arquivo da URL ou do cookie gravados não foi lido.
    ArquivoDaSessao(#[source] ErroArquivoPrivado),
""",
    )
    return trocar(
        t,
        "            | Self::Origem(_)\n",
        "            | Self::OrigemIlegivel(_)\n            | Self::OrigemNaoAutorizada\n",
    )


@ajuste(
    "src/sessao_login/fluxo.rs",
    "OrigemIlegivel e SessaoGravadaInvalida (uma variante só) saem; a recusa do transporte sobe como Transporte",
)
def _(t):
    t = trocar(
        t,
        """        Err(
            crate::arquivo_privado::ErroArquivoPrivado::Ausente
            | crate::arquivo_privado::ErroArquivoPrivado::Vazio,
        ) => Ok(None),
        Err(falha) => Err(SessaoGravadaInvalida::Arquivo(falha).into()),""",
        f"""        Err({ERRO}::Ausente | {ERRO}::Vazio) => Ok(None),
        Err(falha) => Err(ErroLogin::ArquivoDaSessao(falha)),""",
    )
    t = trocar(
        t,
        """    transporte::validar_origem(&url, origem).map_err(SessaoGravadaInvalida::Arquivo)?;
    transporte::validar_cookie(&cookie).map_err(SessaoGravadaInvalida::Arquivo)?;""",
        """    transporte::validar_origem(&url, origem)?;
    transporte::validar_cookie(&cookie)?;""",
    )
    t = trocar(t, ".map_err(OrigemIlegivel::Arquivo)?", ".map_err(ErroLogin::OrigemIlegivel)?", 2)
    t = trocar_re(
        t,
        r"\.ok_or\(OrigemIlegivel::Arquivo\(\s*FalhaTransporte::OrigemNaoAutorizada,?\s*\)\)\?",
        ".ok_or(ErroLogin::OrigemNaoAutorizada)?",
        2,
    )
    return trocar(
        t,
        """
/// O arquivo de origem da instalação ausente, inseguro ou inválido. Tipo próprio da etapa; o nome
/// público é o motivo do transporte (`transparente`).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum OrigemIlegivel {
    Arquivo(#[from] crate::transporte::FalhaTransporte),
}

/// URL ou cookie gravados que o transporte recusa antes da rede. Tipo próprio da etapa; o nome
/// público é o motivo do transporte (`transparente`).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum SessaoGravadaInvalida {
    Arquivo(#[from] crate::transporte::FalhaTransporte),
}
""",
        "",
    )


@ajuste("src/sessao_login/gravacao.rs", "o marcador ilegível leva o ErroArquivoPrivado")
def _(t):
    t = trocar(
        t,
        "Err(crate::arquivo_privado::ErroArquivoPrivado::Ausente) => Ok(None),",
        "Err(ErroArquivoPrivado::Ausente) => Ok(None),",
    )
    t = trocar(
        t,
        "    Ilegivel(#[source] crate::transporte::FalhaTransporte),\n",
        "    Ilegivel(#[source] ErroArquivoPrivado),\n",
    )
    return trocar(
        t,
        "use crate::arquivo_privado::{\n",
        "use crate::arquivo_privado::{\n    ErroArquivoPrivado,\n",
    )


@ajuste(
    "src/documento_html_privado_cli.rs", "a fonte do HTML privado é o arquivo, não o transporte"
)
def _(t):
    return trocar(
        t,
        "    Fonte(#[from] transporte::FalhaTransporte),\n",
        "    Fonte(#[from] crate::arquivo_privado::ErroArquivoPrivado),\n",
    )

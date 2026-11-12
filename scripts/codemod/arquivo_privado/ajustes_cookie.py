"""Ajustes à mão do cookie da sessão, cada um com o motivo. O cookie que a
abertura leu e conferiu chega à escrita como `CookieLido`, sem ser relido:
a variante `Cookie` dos comandos sai (regras 10 a 14) e `ler_cookie` some."""

from ajustes_base import ajuste, trocar, trocar_re


def campo_no_fim(t, struct, campo):
    """Acrescenta `campo` antes da `}` que fecha `pub(crate) struct <struct> {`."""
    inicio = t.find(f"pub(crate) struct {struct} {{\n")
    if inicio < 0 or t.count(f"pub(crate) struct {struct} {{\n") != 1:
        raise ValueError(f"struct {struct} não achada uma vez")
    fim = t.index("\n}\n", inicio)
    return t[:fim] + "\n" + campo + t[fim:]


# --- transporte: o estado do cookie, com um fato por variante ---------------


@ajuste(
    "src/transporte/sessao.rs",
    "CookieDaSessao diz o fato (lido, arquivo ilegível, fora do formato, não aberto); "
    "CookieLido é o cookie conferido; cada leitura da sessão embrulha o erro com o nome do arquivo",
)
def _(t):
    t = trocar(
        t,
        "use super::{FalhaTransporte, validar_cookie, validar_origem};\n",
        "use super::{FalhaTransporte, cookie_no_formato, validar_origem};\n"
        "use crate::arquivo_privado::{self, ErroArquivoPrivado};\n",
    )
    t = trocar(
        t,
        """/// Cookie de um retrato da sessão, já lido do arquivo privado (`0600`,
/// sem link, sintaxe validada) ou a falha da leitura. A falha só aparece
/// onde a requisição pede o cookie, como antes da #222: as recusas offline
/// que vêm antes dela continuam vencendo. Nunca sai no `Debug`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CookieDaSessao(Result<String, FalhaTransporte>);

impl CookieDaSessao {
    /// O chamador deve ter autorizado a URL de destino antes desta função.
    pub(crate) fn ler(cookie_file: &Path) -> Self {
        Self(
            crate::arquivo_privado::ler(cookie_file, 8_192).and_then(|texto| {
                let cookie = texto.trim().to_owned();
                validar_cookie(&cookie)?;
                Ok(cookie)
            }),
        )
    }

    /// Cookie sintético dos testes, validado como o lido do arquivo.
    #[cfg(test)]
    pub(crate) fn sintetico(cookie: &str) -> Self {
        validar_cookie(cookie).unwrap();
        Self(Ok(cookie.to_owned()))
    }

    pub(crate) fn valor(&self) -> Result<&str, FalhaTransporte> {
        self.0.as_deref().map_err(Clone::clone)
    }
}
""",
        """/// Cookie de um retrato da sessão: lido do arquivo privado (`0600`, sem
/// link) e no formato do cabeçalho, ou o fato que o impediu. A falha só
/// aparece onde a requisição pede o cookie, como antes da #222: as recusas
/// offline que vêm antes dela continuam vencendo. Nunca sai no `Debug`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum CookieDaSessao {
    Lido(CookieLido),
    /// O arquivo do cookie não foi lido.
    Ilegivel(ErroArquivoPrivado),
    /// O cookie lido está fora do formato do cabeçalho `Cookie`.
    ForaDoFormato,
    /// A URL do Controle não é da origem autorizada: o cookie nem foi
    /// aberto (a recusa de `validar_origem`).
    NaoAberto(FalhaTransporte),
}

/// O cookie lido e conferido: quem o tem não relê o arquivo nem falha.
/// Só o retrato o cria. Nunca sai no `Debug`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CookieLido(String);

impl std::ops::Deref for CookieLido {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for CookieLido {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CookieLido(<redigido>)")
    }
}

impl CookieLido {
    /// Cookie sintético dos testes, no formato do lido do arquivo.
    #[cfg(test)]
    pub(crate) fn sintetico(cookie: &str) -> Self {
        assert!(cookie_no_formato(cookie));
        Self(cookie.to_owned())
    }
}

impl CookieDaSessao {
    /// O chamador deve ter autorizado a URL de destino antes desta função.
    pub(crate) fn ler(cookie_file: &Path) -> Self {
        match arquivo_privado::ler(cookie_file, 8_192) {
            Ok(texto) if cookie_no_formato(texto.trim()) => {
                Self::Lido(CookieLido(texto.trim().to_owned()))
            }
            Ok(_) => Self::ForaDoFormato,
            Err(erro) => Self::Ilegivel(erro),
        }
    }

    /// Cookie sintético dos testes, validado como o lido do arquivo.
    #[cfg(test)]
    pub(crate) fn sintetico(cookie: &str) -> Self {
        Self::Lido(CookieLido::sintetico(cookie))
    }

    /// O cookie para a requisição; a falha, como o transporte a nomeia.
    pub(crate) fn lido(&self) -> Result<CookieLido, FalhaTransporte> {
        self.valor().map(|cookie| CookieLido(cookie.to_owned()))
    }

    pub(crate) fn valor(&self) -> Result<&str, FalhaTransporte> {
        match self {
            Self::Lido(cookie) => Ok(cookie),
            Self::Ilegivel(erro) => Err(FalhaTransporte::CookieDaSessaoIlegivel(*erro)),
            Self::ForaDoFormato => Err(FalhaTransporte::CookieInvalido),
            Self::NaoAberto(recusa) => Err(recusa.clone()),
        }
    }
}
""",
    )
    t = trocar(
        t,
        """        let url = crate::arquivo_privado::ler(url_file, 8_192)?
            .trim()
            .to_owned();
        let origem = crate::arquivo_privado::ler(origem_file, 512)?
            .trim()
            .to_owned();""",
        """        let url = arquivo_privado::ler(url_file, 8_192)
            .map_err(FalhaTransporte::UrlDoControleIlegivel)?
            .trim()
            .to_owned();
        let origem = arquivo_privado::ler(origem_file, 512)
            .map_err(FalhaTransporte::OrigemIlegivel)?
            .trim()
            .to_owned();""",
    )
    t = trocar(
        t,
        "Err(falha) => CookieDaSessao(Err(falha)),",
        "Err(falha) => CookieDaSessao::NaoAberto(falha),",
    )
    return trocar(
        t,
        """    match crate::arquivo_privado::ler(Path::new(&caminho), 4_096) {
        Ok(texto) => Ok(Some(texto)),
        Err(crate::arquivo_privado::ErroArquivoPrivado::Ausente) => Ok(None),
        Err(falha) => Err(falha),
    }""",
        """    match arquivo_privado::ler(Path::new(&caminho), 4_096) {
        Ok(texto) => Ok(Some(texto)),
        Err(ErroArquivoPrivado::Ausente) => Ok(None),
        Err(falha) => Err(FalhaTransporte::MarcadorDaSessaoIlegivel(falha)),
    }""",
    )


@ajuste(
    "src/transporte.rs", "o formato do cookie é um fato que o retrato guarda sem FalhaTransporte"
)
def _(t):
    return trocar(
        t,
        """pub(crate) fn validar_cookie(cookie: &str) -> Result<(), FalhaTransporte> {
    if cookie.is_empty() || cookie.contains(':') || !cookie.bytes().all(|b| (32..=126).contains(&b))
    {
        return Err(FalhaTransporte::CookieInvalido);
    }
    Ok(())
}""",
        """pub(crate) fn validar_cookie(cookie: &str) -> Result<(), FalhaTransporte> {
    if !cookie_no_formato(cookie) {
        return Err(FalhaTransporte::CookieInvalido);
    }
    Ok(())
}

/// Não vazio, sem `:` e só ASCII visível ou espaço: o valor do cabeçalho.
pub(crate) fn cookie_no_formato(cookie: &str) -> bool {
    !cookie.is_empty() && !cookie.contains(':') && cookie.bytes().all(|b| (32..=126).contains(&b))
}""",
    )


# --- a abertura entrega o cookie conferido ----------------------------------


@ajuste("src/processo_obter/abertura.rs", "a rede da abertura guarda o cookie conferido")
def _(t):
    t = trocar(t, "    pub(super) cookie: String,\n", "    pub(super) cookie: CookieLido,\n")
    return trocar(
        t,
        "use crate::transporte::sessao::CookieDaSessao;\n",
        "use crate::transporte::sessao::{CookieDaSessao, CookieLido};\n",
    )


@ajuste(
    "src/processo_obter/fluxo.rs",
    "abrir_para_escrever lê o cookie uma vez e o entrega em AcoesDeEstado",
)
def _(t):
    t = trocar(
        t,
        "    let cookie = sessao.cookie.valor()?.to_owned();\n",
        "    let cookie = sessao.cookie.lido()?;\n",
    )
    return trocar(
        t,
        "        acompanhar: arvore_lida.acompanhar,\n    };\n    Ok((aberto, acoes))",
        "        acompanhar: arvore_lida.acompanhar,\n        cookie: rede.cookie.clone(),\n    };\n"
        "    Ok((aberto, acoes))",
    )


@ajuste("src/processo_obter/acoes.rs", "o cookie conferido pela abertura segue para a escrita")
def _(t):
    return campo_no_fim(
        t,
        "AcoesDeEstado",
        "    /// O cookie que a abertura leu e conferiu: a escrita não o relê.\n"
        "    pub(crate) cookie: crate::transporte::sessao::CookieLido,",
    )


@ajuste(
    "src/processo_obter/documento.rs",
    "a abertura do documento lê o cookie uma vez e o entrega no documento aberto ou lido",
)
def _(t):
    t = trocar(
        t,
        """    let cookie = sessao
        .cookie
        .valor()
        .map_err(|f| ErroDocumentoObter::Processo(ErroProcessoObter::Transporte(f)))?
        .to_owned();""",
        """    let cookie = sessao
        .cookie
        .lido()
        .map_err(|f| ErroDocumentoObter::Processo(ErroProcessoObter::Transporte(f)))?;""",
    )
    campo = (
        "    /// O cookie que a abertura leu e conferiu: a escrita não o relê.\n"
        "    pub(crate) cookie: crate::transporte::sessao::CookieLido,"
    )
    t = campo_no_fim(t, "DocumentoNaArvoreAberto", campo)
    t = campo_no_fim(t, "DocumentoObtido", campo)
    t = trocar(
        t,
        "            html: lido.html.to_owned(),\n            requisicoes: rede.requisicoes.total_u8(),\n",
        "            html: lido.html.to_owned(),\n            requisicoes: rede.requisicoes.total_u8(),\n"
        "            cookie: rede.cookie.clone(),\n",
    )
    return trocar(
        t,
        "        versoes,\n        propriedades,\n    })\n}",
        "        versoes,\n        propriedades,\n        cookie: rede.cookie.clone(),\n    })\n}",
    )


@ajuste("src/documento_versao_cli.rs", "o documento sintético do teste leva um cookie sintético")
def _(t):
    return trocar_re(
        t,
        r"(\n            propriedades: crate::propriedades_documento::PropriedadesDocumento::NaoPedidas,\n)",
        r"\1            cookie: crate::transporte::sessao::CookieLido::sintetico("
        '"PHPSESSID=SINTETICO"'
        r"),\n",
    )


# --- escrita: ler_cookie e CookieIlegivel saem ------------------------------


@ajuste(
    "src/processo_concluir/escrita.rs",
    "ler_cookie relia o retrato depois da abertura, que já o leu: sai, com CookieIlegivel",
)
def _(t):
    return trocar(
        t,
        """/// O cookie do retrato da sessão (#222), lido do arquivo privado e conferido.
/// O cookie privado da sessão não pôde ser lido, antes de qualquer
/// requisição. Tipo próprio da etapa: os fluxos o recebem por `From` como
/// erro `Cookie`, distinto do transporte das telas.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum CookieIlegivel {
    Arquivo(#[from] FalhaTransporte),
}

pub(crate) fn ler_cookie(sessao: &SessaoDoProcesso<'_>) -> Result<String, CookieIlegivel> {
    Ok(sessao.cookie.valor()?.to_owned())
}

""",
        "",
    )


@ajuste(
    "src/processo_sobrestar.rs",
    "a preparação devolve o cookie da abertura à escrita (a regra 10 não alcança: outra função)",
)
def _(t):
    t = trocar(
        t,
        ".map(|(previa, _)| previa)",
        ".map(|(previa, _, _)| previa)",
    )
    t = trocar(
        t,
        ") -> Result<(Previa, EnderecoDeEscrita), ErroSobrestamento> {",
        ") -> Result<(Previa, EnderecoDeEscrita, crate::transporte::sessao::CookieLido), ErroSobrestamento> {",
    )
    return trocar(
        t,
        "        },\n        endereco,\n    ))\n}",
        "        },\n        endereco,\n        acoes.cookie,\n    ))\n}",
    )


@ajuste("src/processo_sobrestar/fluxo.rs", "o cookie vem da preparação, que abriu o processo")
def _(t):
    t = trocar(
        t,
        "let (mut previa, endereco) = preparar_com_acao(",
        "let (mut previa, endereco, cookie) = preparar_com_acao(",
    )
    return trocar(t, "    let cookie = ler_cookie(sessao)?;\n", "")


@ajuste(
    "src/processo_marcar/escrita.rs",
    "preparar_submissao recebe o cookie da abertura em vez de reler o retrato",
)
def _(t):
    t = trocar(
        t,
        "    rotulo: &Rotulo,\n) -> anyhow::Result<crate::formulario_web::SubmissaoDerivada> {\n"
        "    let cookie = ler_cookie(sessao)?;\n"
        "    let (html, mime) = match obter_tela(rede, &cookie, link) {",
        "    rotulo: &Rotulo,\n    cookie: &str,\n) -> anyhow::Result<crate::formulario_web::SubmissaoDerivada> {\n"
        "    let (html, mime) = match obter_tela(rede, cookie, link) {",
    )
    return trocar(
        t,
        "preparar_submissao(sessao, &mut rede, link.as_str(), unidade, rotulo)?",
        "preparar_submissao(sessao, &mut rede, link.as_str(), unidade, rotulo, &acoes.cookie)?",
    )


@ajuste(
    "src/processo_marcar/catalogo.rs",
    "submissao_cadastro recebe a origem e o cookie da abertura em vez da sessão, que relia o retrato",
)
def _(t):
    t = trocar(
        t,
        """fn submissao_cadastro<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,""",
        """fn submissao_cadastro<V: EnvioDoProcesso>(
    (origem, cookie): (&str, &str),""",
    )
    t = trocar(
        t,
        """    let cookie = ler_cookie(sessao)?;
    let instalacao = Instalacao::interpretar(sessao.origem)""",
        """    let instalacao = Instalacao::interpretar(origem)""",
    )
    t = trocar(
        t,
        "destino_cadastro(rede, &cookie, link, &instalacao, id)?",
        "destino_cadastro(rede, cookie, link, &instalacao, id)?",
    )
    t = trocar(
        t, "match obter_tela(rede, &cookie, &url) {", "match obter_tela(rede, cookie, &url) {"
    )
    return trocar(
        t,
        "    let submissao = submissao_cadastro(\n        sessao,\n",
        "    let submissao = submissao_cadastro(\n        (sessao.origem, &acoes.cookie),\n",
    )


@ajuste(
    "src/unidade_trocar.rs",
    "a troca lê o cookie uma vez e o entrega no resultado, para regravar a sessão",
)
def _(t):
    t = trocar(
        t,
        "    let cookie = sessao.cookie.valor()?;\n",
        "    let cookie = &sessao.cookie.lido()?;\n",
    )
    t = trocar(
        t,
        "        controle_novo: None,\n    };",
        "        controle_novo: None,\n        cookie: cookie.clone(),\n    };",
    )
    return campo_no_fim(
        t,
        "Troca",
        "    /// O cookie que a troca leu e conferiu: a regravação não o relê.\n"
        "    pub(crate) cookie: crate::transporte::sessao::CookieLido,",
    )


@ajuste(
    "src/unidade_trocar_cli.rs",
    "aplicar_troca recebe o cookie da troca; FalhaAplicarTroca fica com uma variante só e sai",
)
def _(t):
    t = trocar(
        t,
        "    pub(crate) causa: FalhaAplicarTroca,\n",
        "    pub(crate) causa: crate::sessao_login::ErroLogin,\n",
    )
    t = trocar_re(
        t,
        r"/// Causas diferentes permanecem tipadas\.\n#\[derive\(Debug, PartialEq, Eq, thiserror::Error\)\]\n"
        r"#\[error\(\"\{self:\?\}\"\)\]\npub\(crate\) enum FalhaAplicarTroca \{\n[^}]*\}\n\n",
        "",
    )
    t = trocar(
        t,
        "    cookie: &CookieDaSessao,\n    novo: &ControleLido,\n) -> Result<Option<usize>, FalhaAplicarTroca> {\n"
        "    let cookie = cookie.valor().map_err(FalhaAplicarTroca::Cookie)?;\n"
        "    crate::sessao_login::regravar_url_do_controle(arquivos, &novo.url, cookie)\n"
        "        .map_err(FalhaAplicarTroca::Gravacao)?;\n",
        "    cookie: &str,\n    novo: &ControleLido,\n) -> Result<Option<usize>, crate::sessao_login::ErroLogin> {\n"
        "    crate::sessao_login::regravar_url_do_controle(arquivos, &novo.url, cookie)?;\n",
    )
    return trocar(
        t,
        "aplicar_troca(arquivos, indice, instalacao, sessao.cookie(), novo)",
        "aplicar_troca(arquivos, indice, instalacao, &troca.cookie, novo)",
    )


@ajuste(
    "src/documento_salvar_cli/remoto.rs",
    "o editor usa o cookie do documento lido; a variante Cookie, que nenhum caminho produz, sai",
)
def _(t):
    t = trocar(t, "    Cookie(#[source] crate::transporte::FalhaTransporte),\n", "")
    return trocar(
        t,
        """    let cookie = match sessao.cookie.valor() {
        Ok(cookie) => cookie,
        Err(falha) => return Resultado::Recusado(ErroSalvarNoSei::Cookie(falha)),
    };
""",
        "    let cookie: &str = &lido.cookie;\n",
    )


@ajuste(
    "src/processo_criar/erro_fluxo.rs",
    "o cookie é lido aqui pela primeira vez: cada fato do retrato vira variante própria",
)
def _(t):
    t = trocar(
        t,
        """    /// Cookie da sessão recusado pelo transporte, antes da rede.
    CookieDaSessaoIlegivel(#[source] FalhaTransporte),
""",
        """    /// O arquivo do cookie da sessão não foi lido, antes da rede.
    CookieDaSessaoIlegivel(#[source] ErroArquivoPrivado),
    /// O cookie da sessão está fora do formato do cabeçalho `Cookie`.
    CookieForaDoFormato,
    /// A URL do Controle não é da origem autorizada: o cookie não foi aberto.
    ControleForaDaOrigem(#[source] FalhaTransporte),
""",
    )
    return trocar(
        t,
        "use crate::transporte::FalhaTransporte;\n",
        "use crate::arquivo_privado::ErroArquivoPrivado;\nuse crate::transporte::FalhaTransporte;\n",
    )


@ajuste(
    "src/processo_criar/fluxo.rs",
    "o estado do retrato vira o fato de ErroCriar, sem FalhaTransporte no meio",
)
def _(t):
    return trocar(
        t,
        "    let cookie_valor = cookie.valor().map_err(ErroCriar::CookieDaSessaoIlegivel)?;\n",
        """    let cookie_valor: &str = match cookie {
        CookieDaSessao::Lido(lido) => lido,
        CookieDaSessao::Ilegivel(erro) => return Err(ErroCriar::CookieDaSessaoIlegivel(*erro)),
        CookieDaSessao::ForaDoFormato => return Err(ErroCriar::CookieForaDoFormato),
        CookieDaSessao::NaoAberto(recusa) => {
            return Err(ErroCriar::ControleForaDaOrigem(recusa.clone()));
        }
    };
""",
    )

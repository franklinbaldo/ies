//! Exclusao do prazo pela lista da unidade, nunca pelo cadastro da barra.
//! O unico POST e reservado ao anti-IES de teste; release recusa antes da rede.
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroChaveDaConsulta, ErroConsultaDoControlador, menciona_acao,
};
use crate::formulario_web::derivar_submissao_com_action;
use crate::url_observada::UrlHttps;

const EXCLUIR_PRAZO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ControlePrazoExcluir,
    formulario: "frmControlePrazoLista",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnInfraItemId"],
    campos_ocultos_reescritos: &["hdnInfraItemId"],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::ControleDePrazo,
    },
};

struct ExclusaoPreparada {
    link_lista: String,
    html_lista: crate::tela_html::HtmlServido,
    mime: String,
    id_controle: String,
    id_procedimento: String,
    nup: String,
    /// Unidade da sessão lida na abertura; a lista a publica em
    /// `lnkInfraUnidade` e o primitivo de formulário a confere.
    unidade: String,
}

/// Por que o link da lista de prazos do Controle não foi aceito.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLinkDaLista {
    /// O HTML do Controle não pôde ser interpretado.
    ControleIlegivel,
    /// A origem da sessão não é uma instalação reconhecida.
    OrigemNaoInterpretada,
    /// Nenhum link do Controle aponta para `controle_prazo_listar`.
    LinkDaListaAusente,
    /// Mais de um link do Controle aponta para a lista de prazos.
    LinkDaListaRepetido,
    /// O link da lista está fora da forma segura (caminho, host, ação única ou hash).
    LinkDaListaForaDaForma,
    /// A consulta do link não se deixa ler em pares.
    ConsultaDoLinkIlegivel(#[source] ErroConsultaDoControlador),
    /// O link não traz exatamente um `infra_hash`.
    InfraHashDoLink(#[source] ErroChaveDaConsulta),
}

/// Lê o href realmente publicado na pagina do Controle, sem fabricar token.
pub(super) fn link_da_lista(
    html: &crate::tela_html::HtmlServido,
    origem: &str,
) -> Result<String, ErroLinkDaLista> {
    let dom = crate::tela_html::interpretar(html);
    let instalacao =
        Instalacao::interpretar(origem).ok_or(ErroLinkDaLista::OrigemNaoInterpretada)?;
    let mut encontrados = Vec::new();
    for ancora in dom
        .query_selector("a")
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(dom.parser()).and_then(tl::Node::as_tag))
    {
        let Some(bruto) = crate::html_atributos::valor_do_atributo(ancora, "href")
            .map_err(|_| ErroLinkDaLista::ControleIlegivel)?
        else {
            continue;
        };
        let href = crate::formulario_web::decodificar_atributo(&bruto);
        if !menciona_acao(&href, AcaoDoSei::ControlePrazoListar) {
            continue;
        }
        if href.starts_with('/') || href.contains("://") || href.contains("..") {
            return Err(ErroLinkDaLista::LinkDaListaForaDaForma);
        }
        let url = format!("{origem}/{href}");
        let alvo = UrlHttps::interpretar(&url).ok_or(ErroLinkDaLista::LinkDaListaForaDaForma)?;
        if !instalacao.autoriza_controlador(alvo.host, alvo.caminho) {
            return Err(ErroLinkDaLista::LinkDaListaForaDaForma);
        }
        let consulta = alvo
            .consulta
            .ok_or(ErroLinkDaLista::LinkDaListaForaDaForma)?;
        let consulta = ConsultaDoControlador::ler(consulta)
            .map_err(ErroLinkDaLista::ConsultaDoLinkIlegivel)?;
        if consulta.acao() != Ok(AcaoDoSei::ControlePrazoListar) {
            return Err(ErroLinkDaLista::LinkDaListaForaDaForma);
        }
        let hash = consulta
            .unico("infra_hash")
            .map_err(ErroLinkDaLista::InfraHashDoLink)?;
        if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ErroLinkDaLista::LinkDaListaForaDaForma);
        }
        encontrados.push(url);
    }
    match encontrados.as_slice() {
        [] => Err(ErroLinkDaLista::LinkDaListaAusente),
        [link] => Ok(link.clone()),
        [_, ..] => Err(ErroLinkDaLista::LinkDaListaRepetido),
    }
}

/// A lista de prazos fora da forma observada.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroListaDePrazos {
    FormularioDaListaNaoLocalizado(#[source] crate::formulario_web::ErroSubmissaoFormulario),
    AtributoDaLista(#[source] crate::html_atributos::ErroAtributoHtml),
    AcaoExcluirForaDaForma,
    IdDoControleNaoNumerico,
    /// Duas linhas com `acaoExcluir` para o mesmo NUP.
    ControlesRepetidosParaOProcesso,
}

/// Lê uma funcao de linha observada sem executar o JavaScript.
/// Ausencia de acao ao confirmar a exclusao significa ausencia da linha.
pub(super) fn id_na_lista(
    html: &crate::tela_html::HtmlServido,
    nup: &str,
) -> Result<Option<String>, ErroListaDePrazos> {
    let dom = crate::tela_html::interpretar(html);
    crate::formulario_web::localizar_formulario(&dom, "frmControlePrazoLista")
        .map_err(ErroListaDePrazos::FormularioDaListaNaoLocalizado)?;
    let mut encontrado = None;
    for ancora in dom
        .query_selector("a")
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(dom.parser()).and_then(tl::Node::as_tag))
    {
        let Some(acao) = crate::html_atributos::valor_do_atributo(ancora, "onclick")
            .map_err(ErroListaDePrazos::AtributoDaLista)?
        else {
            continue;
        };
        let Some(resto) = acao.trim().strip_prefix("acaoExcluir('") else {
            continue;
        };
        let Some((id, numero)) = resto.split_once("','") else {
            return Err(ErroListaDePrazos::AcaoExcluirForaDaForma);
        };
        let Some(numero) = numero
            .strip_suffix("');")
            .or_else(|| numero.strip_suffix("')"))
        else {
            return Err(ErroListaDePrazos::AcaoExcluirForaDaForma);
        };
        if numero != nup {
            continue;
        }
        if id.is_empty() || !id.bytes().all(|c| c.is_ascii_digit()) {
            return Err(ErroListaDePrazos::IdDoControleNaoNumerico);
        }
        if encontrado.replace(id.to_owned()).is_some() {
            return Err(ErroListaDePrazos::ControlesRepetidosParaOProcesso);
        }
    }
    Ok(encontrado)
}

/// A URL literal do script passa pela mesma validacao da origem e do hash
/// usada pelo primitivo de formulario, nao por um executor de JavaScript.
/// A `UnB` publica parametros entre `acao` e `infra_hash` (cassette
/// processo-prazo-excluir-unb, interacao 13; live de 10/11/2026):
/// `controlador.php?acao=controle_prazo_excluir&acao_origem=…&infra_hash=…`.
/// Exige uma unica URL distinta, terminada por `infra_hash` de 64 hex.
pub(super) fn action_de_exclusao(html: &str) -> Result<String, ErroAcaoDeExclusao> {
    let prefixo = "controlador.php?acao=controle_prazo_excluir";
    let mut encontradas: Vec<&str> = Vec::new();
    for (inicio, _) in html.match_indices(prefixo) {
        let (_, resto) = html.split_at(inicio);
        let fim = resto
            .find(['\'', '"', ' ', '<', '>', '\n'])
            .ok_or(ErroAcaoDeExclusao::ForaDaForma)?;
        let (url, _) = resto.split_at(fim);
        if !encontradas.contains(&url) {
            encontradas.push(url);
        }
    }
    let url = match encontradas.as_slice() {
        [] => return Err(ErroAcaoDeExclusao::NaoPublicada),
        [url] => *url,
        [_, ..] => return Err(ErroAcaoDeExclusao::Repetida),
    };
    let consulta = url
        .strip_prefix(prefixo)
        .ok_or(ErroAcaoDeExclusao::ForaDaForma)?;
    if !(consulta.is_empty() || consulta.starts_with('&')) {
        return Err(ErroAcaoDeExclusao::ForaDaForma);
    }
    let hash = url
        .rsplit_once("&infra_hash=")
        .ok_or(ErroAcaoDeExclusao::ForaDaForma)?
        .1;
    if hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(url.to_owned())
    } else {
        Err(ErroAcaoDeExclusao::ForaDaForma)
    }
}

/// Por que a ação de exclusão publicada pela lista não pôde ser derivada.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAcaoDeExclusao {
    /// A lista não publica nenhuma ação de exclusão do controle de prazo.
    NaoPublicada,
    /// A lista publica mais de uma ação distinta de exclusão.
    Repetida,
    /// A ação publicada não tem a consulta ou o `infra_hash` da forma observada.
    ForaDaForma,
}

fn escrever_e_verificar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    (pedido, indice): (&crate::processo_obter::PedidoDoProcesso<'_>, &IndiceLocal),
    dados: &ExclusaoPreparada,
    rede: &mut Contador<'_, V>,
    cookie: &str,
) -> Result<(), FalhaDoPrazo> {
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(FalhaDoPrazo::ConfirmacaoSemFormularioSeguro)?;
    let codificacao =
        codificacao_da_tela(&dados.mime).ok_or(FalhaDoPrazo::ConfirmacaoSemFormularioSeguro)?;
    let url_declarada =
        action_de_exclusao(&dados.html_lista).map_err(FalhaDoPrazo::AcaoDeExclusao)?;
    let servida = TelaServida {
        url: &dados.link_lista,
        html: &dados.html_lista,
        codificacao,
    };
    let submissao = derivar_submissao_com_action(
        &EXCLUIR_PRAZO,
        &instalacao,
        &servida,
        Some(dados.unidade.as_str()),
        &[("hdnInfraItemId", dados.id_controle.as_str())],
        Some(&url_declarada),
    )
    .map_err(FalhaDoPrazo::SubmissaoRecusada)?;
    let mut enviou = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |post, c| {
        enviou = true;
        rede.post(post, c)
    })
    .map_err(|erro| {
        if enviou {
            FalhaDoPrazo::NaoConfirmado(Some(erro))
        } else {
            FalhaDoPrazo::SubmissaoRecusada(erro)
        }
    })?;
    match resposta {
        RespostaDoEnvio::Redirecionamento(redir)
            if redir.localizacao().is_some_and(|local| {
                local.contains("acao=controle_prazo_listar")
                    && local.contains("acao_origem=controle_prazo_excluir")
            }) => {}
        RespostaDoEnvio::Html(_) | RespostaDoEnvio::Redirecionamento(_) => {
            return Err(FalhaDoPrazo::NaoConfirmado(None));
        }
    }
    let (lista_nova, _) =
        obter_tela(rede, cookie, &dados.link_lista).map_err(FalhaDoPrazo::TelaRelidaNaoObtida)?;
    // A mesma linha do NUP precisa desaparecer da lista relida.
    match id_na_lista(&lista_nova, &dados.nup) {
        Ok(None) => {}
        Ok(Some(_)) => return Err(FalhaDoPrazo::NaoConfirmado(None)),
        Err(erro) => return Err(FalhaDoPrazo::ListaRelidaIlegivel(erro)),
    }
    // O GET do formulario de definicao e testemunha independente da lista.
    // O link guardado leva `id_controle_prazo` do controle excluido (cassette
    // processo-prazo-excluir-unb, interacoes 11 e 19): reabrir o processo e
    // usar o link novo da barra, como a leitura de `ies processo prazo`.
    let (_, acoes) = abrir_para_escrever(sessao, pedido, indice, rede)
        .map_err(FalhaDoPrazo::ProcessoRelidoNaoAbriu)?;
    let Some(link_novo) = acoes
        .prazo
        .map_err(FalhaDoPrazo::LinkDePrazoRelidoIlegivel)?
    else {
        return Err(FalhaDoPrazo::NaoConfirmado(None));
    };
    let (formulario, _) =
        obter_tela(rede, cookie, link_novo.as_str()).map_err(FalhaDoPrazo::TelaRelidaNaoObtida)?;
    let prazo =
        ler_tela(&formulario, &dados.id_procedimento).map_err(FalhaDoPrazo::PrazoRelidoIlegivel)?;
    if prazo.estado != EstadoDoPrazo::SemPrazo {
        return Err(FalhaDoPrazo::NaoConfirmado(None));
    }
    Ok(())
}

fn publicar_exclusao(
    cli: &Cli,
    nup: &NumeroProcesso,
    confirmar: bool,
    (aberto, indice): (&crate::processo_obter::ProcessoAberto, &IndiceLocal),
    requisicoes: u8,
) {
    // Na prévia o prazo atual não é lido: a chave `atual` sai, sem null.
    let mut resultado = json!({
        "nup": nup.as_str(),
        "id_procedimento": aberto.arvore.processo.id,
        "origem": "web",
        "solicitado": {"excluir": true},
        "confirmado": confirmar,
        "requisicoes": requisicoes,
    });
    if let Some(objeto) = resultado.as_object_mut() {
        if let Some(unidade) = &aberto.unidade {
            objeto.insert("unidade".to_owned(), json!(unidade));
        }
        if confirmar {
            objeto.insert("atual".to_owned(), json!({"tipo": "sem_prazo"}));
        }
    }
    if cli.json {
        crate::indice_local::anexar_avisos(&mut resultado, indice);
        println!("{resultado}");
    } else if confirmar {
        println!(
            "Prazo do processo {} excluido e verificado por leitura.",
            nup.as_str()
        );
    } else {
        println!(
            "Previa: excluir o controle de prazo de {}. Sem escrita.",
            nup.as_str()
        );
    }
}

/// Executa BDD-272-03 na CLI real, incluindo a previa sem qualquer POST.
pub(super) fn executar(
    cli: &Cli,
    nup: &NumeroProcesso,
    confirmar: bool,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
) -> anyhow::Result<()> {
    let mut envio = EnvioHttps;
    let mut rede = Contador::novo(&mut envio);
    let leitura = leitura(nup, cli.unidade.as_deref(), true);
    let (aberto, acoes) = abrir_para_escrever(sessao, &leitura, indice, &mut rede)?;
    if acoes.prazo?.is_none() {
        return Err(FalhaDoPrazo::LeituraIndependenteNaoOferecida.into());
    }
    let cookie = ler_cookie(sessao)?;
    let (controle, _) = obter_tela(&mut rede, &cookie, sessao.url_do_controle)?;
    let link_lista =
        link_da_lista(&controle, sessao.origem).map_err(FalhaDoPrazo::LinkDaListaDePrazos)?;
    let (html_lista, mime) = obter_tela(&mut rede, &cookie, &link_lista)?;
    let id_controle = match id_na_lista(&html_lista, nup.as_str()) {
        Ok(Some(id)) => id,
        Ok(None) => return Err(FalhaDoPrazo::ProcessoForaDaListaDePrazos.into()),
        Err(erro) => return Err(FalhaDoPrazo::ListaDePrazos(erro).into()),
    };
    let unidade = aberto
        .unidade
        .clone()
        .ok_or(FalhaDoPrazo::UnidadeDaSessaoNaoPublicada)?;
    let preparada = ExclusaoPreparada {
        link_lista,
        html_lista,
        mime,
        id_controle,
        id_procedimento: aberto.arvore.processo.id.clone(),
        nup: nup.as_str().to_owned(),
        unidade,
    };
    if confirmar {
        escrever_e_verificar(sessao, (&leitura, indice), &preparada, &mut rede, &cookie)?;
    }
    publicar_exclusao(cli, nup, confirmar, (&aberto, indice), rede.feitas());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ErroAcaoDeExclusao, ErroLinkDaLista, action_de_exclusao, link_da_lista};

    fn hash() -> String {
        "a".repeat(64)
    }

    #[test]
    fn lista_da_unb_com_acao_origem_publica_a_acao_de_exclusao() {
        let url = format!(
            "controlador.php?acao=controle_prazo_excluir&acao_origem=controle_prazo_listar&infra_sistema=100000100&infra_unidade_atual=110000847&infra_hash={}",
            hash()
        );
        // A lista repete a mesma URL (duas ocorrências idênticas no script).
        let html = format!("<script>var a='{url}';\nvar b=\"{url}\";</script>");
        assert_eq!(action_de_exclusao(&html), Ok(url));
    }

    #[test]
    fn forma_curta_continua_aceita_e_urls_distintas_sao_recusadas() {
        let curta = format!(
            "controlador.php?acao=controle_prazo_excluir&infra_hash={}",
            hash()
        );
        assert_eq!(action_de_exclusao(&format!("'{curta}'")), Ok(curta.clone()));
        let outra = format!(
            "controlador.php?acao=controle_prazo_excluir&acao_origem=x&infra_hash={}",
            "b".repeat(64)
        );
        assert_eq!(
            action_de_exclusao(&format!("'{curta}' '{outra}'")),
            Err(ErroAcaoDeExclusao::Repetida)
        );
        let sem_hash = "'controlador.php?acao=controle_prazo_excluir&acao_origem=x'";
        assert_eq!(
            action_de_exclusao(sem_hash),
            Err(ErroAcaoDeExclusao::ForaDaForma)
        );
        let colada = format!(
            "'controlador.php?acao=controle_prazo_excluirX&infra_hash={}'",
            hash()
        );
        assert_eq!(
            action_de_exclusao(&colada),
            Err(ErroAcaoDeExclusao::ForaDaForma)
        );
    }

    fn controle_com(href: &str) -> String {
        format!("<html><body><a href=\"{href}\">Prazos</a></body></html>")
    }

    #[test]
    fn link_da_lista_aceita_a_forma_publicada() {
        let href = format!(
            "controlador.php?acao=controle_prazo_listar&infra_sistema=1&infra_hash={}",
            hash()
        );
        let link = link_da_lista(
            &crate::tela_html::HtmlServido::de_teste(&controle_com(&href)),
            "https://sei.exemplo/sei",
        );
        assert_eq!(link.ok(), Some(format!("https://sei.exemplo/sei/{href}")));
    }

    #[test]
    fn link_da_lista_com_duas_acoes_e_recusado() {
        let href = format!(
            "controlador.php?acao=outra&acao=controle_prazo_listar&infra_hash={}",
            hash()
        );
        let erro = link_da_lista(
            &crate::tela_html::HtmlServido::de_teste(&controle_com(&href)),
            "https://sei.exemplo/sei",
        )
        .err();
        assert!(
            matches!(erro, Some(ErroLinkDaLista::LinkDaListaForaDaForma)),
            "{erro:?}"
        );
    }
}

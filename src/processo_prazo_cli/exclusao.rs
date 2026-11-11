//! Exclusao do prazo pela lista da unidade, nunca pelo cadastro da barra.
//! O unico POST e reservado ao anti-IES de teste; release recusa antes da rede.
use super::*;
use crate::formulario_web::derivar_submissao_com_action;
use crate::url_observada::UrlHttps;

const EXCLUIR_PRAZO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "controle_prazo_excluir",
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
    html_lista: String,
    mime: String,
    id_controle: String,
    id_procedimento: String,
    nup: String,
    /// Unidade da sessão lida na abertura; a lista a publica em
    /// `lnkInfraUnidade` e o primitivo de formulário a confere.
    unidade: String,
}

/// Lê o href realmente publicado na pagina do Controle, sem fabricar token.
fn link_da_lista(html: &str, origem: &str) -> Option<String> {
    let dom = tl::parse(html, tl::ParserOptions::default()).ok()?;
    let instalacao = Instalacao::interpretar(origem)?;
    let mut encontrados = Vec::new();
    for ancora in dom
        .query_selector("a")
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(dom.parser()).and_then(tl::Node::as_tag))
    {
        let Some(bruto) = crate::html_atributos::valor_do_atributo(ancora, "href")
            .ok()
            .flatten()
        else {
            continue;
        };
        let Some(href) = crate::formulario_web::decodificar_atributo(&bruto) else {
            continue;
        };
        if !href
            .split(['?', '&'])
            .any(|parte| parte == "acao=controle_prazo_listar")
        {
            continue;
        }
        if href.starts_with('/') || href.contains("://") || href.contains("..") {
            return None;
        }
        let url = format!("{origem}/{href}");
        let alvo = UrlHttps::interpretar(&url)?;
        if !instalacao.autoriza_controlador(alvo.host, alvo.caminho) {
            return None;
        }
        let consulta = alvo.consulta?;
        let hashes: Vec<_> = consulta
            .split('&')
            .filter_map(|parte| parte.strip_prefix("infra_hash="))
            .collect();
        if hashes.len() != 1
            || hashes[0].len() != 64
            || !hashes[0].bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return None;
        }
        encontrados.push(url);
    }
    (encontrados.len() == 1).then(|| encontrados.remove(0))
}

/// Lê uma funcao de linha observada sem executar o JavaScript.
/// Ausencia de acao ao confirmar a exclusao significa ausencia da linha.
fn id_na_lista(html: &str, nup: &str) -> Result<Option<String>, ()> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ())?;
    crate::formulario_web::localizar_formulario(&dom, "frmControlePrazoLista").map_err(|_| ())?;
    let mut encontrado = None;
    for ancora in dom
        .query_selector("a")
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(dom.parser()).and_then(tl::Node::as_tag))
    {
        let Some(acao) =
            crate::html_atributos::valor_do_atributo(ancora, "onclick").map_err(|_| ())?
        else {
            continue;
        };
        let Some(resto) = acao.trim().strip_prefix("acaoExcluir('") else {
            continue;
        };
        let Some((id, numero)) = resto.split_once("','") else {
            return Err(());
        };
        let Some(numero) = numero
            .strip_suffix("');")
            .or_else(|| numero.strip_suffix("')"))
        else {
            return Err(());
        };
        if numero != nup {
            continue;
        }
        if id.is_empty()
            || !id.bytes().all(|c| c.is_ascii_digit())
            || encontrado.replace(id.to_owned()).is_some()
        {
            return Err(());
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
fn action_de_exclusao(html: &str) -> Option<String> {
    let prefixo = "controlador.php?acao=controle_prazo_excluir";
    let mut encontradas: Vec<&str> = Vec::new();
    for (inicio, _) in html.match_indices(prefixo) {
        let resto = &html[inicio..];
        let fim = resto.find(['\'', '"', ' ', '<', '>', '\n'])?;
        let url = &resto[..fim];
        if !encontradas.contains(&url) {
            encontradas.push(url);
        }
    }
    let [url] = encontradas.as_slice() else {
        return None;
    };
    let consulta = url.strip_prefix(prefixo)?;
    if !(consulta.is_empty() || consulta.starts_with('&')) {
        return None;
    }
    let hash = url.rsplit_once("&infra_hash=")?.1;
    (hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit())).then(|| (*url).to_owned())
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
        action_de_exclusao(&dados.html_lista).ok_or(FalhaDoPrazo::AcaoDeExclusaoNaoPublicada)?;
    let servida = TelaServida {
        url: &dados.link_lista,
        html: &dados.html_lista,
        codificacao,
    };
    let submissao = derivar_submissao_com_action(
        &EXCLUIR_PRAZO,
        &instalacao,
        &servida,
        &dados.unidade,
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
    let (lista_nova, _) = obter_tela(rede, cookie, &dados.link_lista)
        .map_err(|_| FalhaDoPrazo::NaoConfirmado(None))?;
    // A mesma linha do NUP precisa desaparecer da lista relida.
    match id_na_lista(&lista_nova, &dados.nup) {
        Ok(None) => {}
        Ok(Some(_)) | Err(()) => return Err(FalhaDoPrazo::NaoConfirmado(None)),
    }
    // O GET do formulario de definicao e testemunha independente da lista.
    // O link guardado leva `id_controle_prazo` do controle excluido (cassette
    // processo-prazo-excluir-unb, interacoes 11 e 19): reabrir o processo e
    // usar o link novo da barra, como a leitura de `ies processo prazo`.
    let (_, acoes) = abrir_para_escrever(sessao, pedido, indice, rede)
        .map_err(|_| FalhaDoPrazo::NaoConfirmado(None))?;
    let Ok(Some(link_novo)) = acoes.prazo else {
        return Err(FalhaDoPrazo::NaoConfirmado(None));
    };
    let (formulario, _) = obter_tela(rede, cookie, link_novo.as_str())
        .map_err(|_| FalhaDoPrazo::NaoConfirmado(None))?;
    let prazo = ler_tela(&formulario, &dados.id_procedimento)
        .map_err(|_| FalhaDoPrazo::NaoConfirmado(None))?;
    if prazo.estado != EstadoDoPrazo::SemPrazo {
        return Err(FalhaDoPrazo::NaoConfirmado(None));
    }
    Ok(())
}

fn publicar_exclusao(
    cli: &Cli,
    nup: &NumeroProcesso,
    confirmar: bool,
    aberto: &crate::processo_obter::ProcessoAberto,
    requisicoes: u8,
) {
    let resultado = json!({
        "nup": nup.as_str(),
        "id_procedimento": aberto.arvore.processo.id,
        "unidade": aberto.unidade,
        "origem": "web",
        "atual": if confirmar { json!({"tipo": "sem_prazo"}) } else { Value::Null },
        "solicitado": {"excluir": true},
        "confirmado": confirmar,
        "requisicoes": requisicoes,
    });
    if cli.json {
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
    let Ok(Some(_)) = acoes.prazo else {
        return Err(FalhaDoPrazo::LeituraIndependenteNaoOferecida.into());
    };
    let cookie = ler_cookie(sessao)?;
    let (controle, _) = obter_tela(&mut rede, &cookie, sessao.url_do_controle)?;
    let link_lista =
        link_da_lista(&controle, sessao.origem).ok_or(FalhaDoPrazo::ListaDePrazosNaoPublicada)?;
    let (html_lista, mime) = obter_tela(&mut rede, &cookie, &link_lista)?;
    let id_controle = match id_na_lista(&html_lista, nup.as_str()) {
        Ok(Some(id)) => id,
        Ok(None) => return Err(FalhaDoPrazo::ProcessoForaDaListaDePrazos.into()),
        Err(()) => return Err(FalhaDoPrazo::ControleNaoUnicoNaLista.into()),
    };
    let preparada = ExclusaoPreparada {
        link_lista,
        html_lista,
        mime,
        id_controle,
        id_procedimento: aberto.arvore.processo.id.clone(),
        nup: nup.as_str().to_owned(),
        unidade: aberto.unidade.clone().unwrap_or_default(),
    };
    if confirmar {
        escrever_e_verificar(sessao, (&leitura, indice), &preparada, &mut rede, &cookie)?;
    }
    publicar_exclusao(cli, nup, confirmar, &aberto, rede.feitas());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::action_de_exclusao;

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
        assert_eq!(action_de_exclusao(&html), Some(url));
    }

    #[test]
    fn forma_curta_continua_aceita_e_urls_distintas_sao_recusadas() {
        let curta = format!(
            "controlador.php?acao=controle_prazo_excluir&infra_hash={}",
            hash()
        );
        assert_eq!(
            action_de_exclusao(&format!("'{curta}'")),
            Some(curta.clone())
        );
        let outra = format!(
            "controlador.php?acao=controle_prazo_excluir&acao_origem=x&infra_hash={}",
            "b".repeat(64)
        );
        assert_eq!(action_de_exclusao(&format!("'{curta}' '{outra}'")), None);
        let sem_hash = "'controlador.php?acao=controle_prazo_excluir&acao_origem=x'";
        assert_eq!(action_de_exclusao(sem_hash), None);
        let colada = format!(
            "'controlador.php?acao=controle_prazo_excluirX&infra_hash={}'",
            hash()
        );
        assert_eq!(action_de_exclusao(&colada), None);
    }
}

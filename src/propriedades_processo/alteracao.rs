//! Prévia da alteração exclusiva de especificação (#877).
//! Capacidade experimental: nenhuma submissão cadastral é enviada.
use super::tela::{ALTERAR_ESPECIFICACAO, pares_de_especificacao};
use super::{ErroPropriedades, FonteDasPropriedades, PropriedadesLidas, ler_propriedades};
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::{conferir_endereco_relativo, ler_link_das_propriedades};
use crate::arvore_processo::endereco::EnderecoRecusado;
use crate::formulario_web::{ErroSubmissaoFormulario, TelaServida, derivar_submissao};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, leitura, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum ErroAlterar {
    OrigemNaoAutorizada,
    Abertura(#[from] ErroProcessoObter),
    Propriedades(#[from] ErroPropriedades),
    AlteracaoNaoOferecida,
    Tela(#[from] FalhaDaTela),
    CharsetNaoSuportado,
    Formulario(#[from] ErroSubmissaoFormulario),
    ActionForaDaInstalacao,
    ActionRecusada(#[from] EnderecoRecusado),
    EscritaAindaNaoValidada,
}

fn preparar(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    nup: &str,
    id: &str,
    pedida: &str,
    unidade: Option<&str>,
) -> Result<PropriedadesLidas, ErroAlterar> {
    let antes = ler_propriedades(tela.html, nup, id, FonteDasPropriedades::Alterar)?;
    let pares = pares_de_especificacao(tela.html, pedida)?;
    let refs: Vec<_> = pares.iter().map(|(n, v)| (*n, v.as_str())).collect();
    let submissao = derivar_submissao(&ALTERAR_ESPECIFICACAO, instalacao, tela, unidade, &refs)?;
    let action = submissao
        .url()
        .strip_prefix(&format!("{}/", instalacao.url_base()))
        .ok_or(ErroAlterar::ActionForaDaInstalacao)?;
    conferir_endereco_relativo(instalacao, action, AcaoDoSei::ProcedimentoAlterar, id)?;
    Ok(antes)
}

struct Previa {
    id: String,
    anterior: String,
    requisicoes: u8,
}

fn prever<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    numero: &NumeroProcesso,
    pedida: &str,
    unidade: Option<&str>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, ErroAlterar> {
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(ErroAlterar::OrigemNaoAutorizada)?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) =
        abrir_para_escrever(sessao, &leitura(numero, unidade, true), indice, &mut rede)?;
    let id = aberto.arvore.processo.id;
    let link = ler_link_das_propriedades(&instalacao, &acoes.html_da_arvore, &id)?
        .filter(|link| link.fonte == FonteDasPropriedades::Alterar)
        .ok_or(ErroAlterar::AlteracaoNaoOferecida)?;
    let (html, mime) = obter_tela(&mut rede, &acoes.cookie, link.as_str())?;
    let codificacao = crate::pagina_do_controle::codificacao_da_tela(&mime)
        .ok_or(ErroAlterar::CharsetNaoSuportado)?;
    let tela = TelaServida {
        url: link.as_str(),
        html: &html,
        codificacao,
    };
    let antes = preparar(
        &instalacao,
        &tela,
        numero.as_str(),
        &id,
        pedida,
        aberto.unidade.as_deref(),
    )?;
    Ok(Previa {
        id,
        anterior: antes.especificacao,
        requisicoes: rede.feitas(),
    })
}

pub(crate) fn tentar_alterar(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Alterar {
            numero,
            especificacao,
            confirmar,
        }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    if *confirmar {
        return Some(Err(ErroAlterar::EscritaAindaNaoValidada.into()));
    }
    Some(crate::processo_concluir_cli::com_sessao(cli, |sessao, indice| {
        let previa = prever(
            sessao,
            numero,
            especificacao,
            cli.unidade.as_deref(),
            indice,
            &mut EnvioHttps,
        )?;
        let necessaria = previa.anterior != *especificacao;
        if cli.json {
            let mut valor = json!({
                "numero": numero.as_str(),
                "id_procedimento": previa.id,
                "especificacao_anterior": previa.anterior,
                "especificacao_pedida": especificacao,
                "alteracao_necessaria": necessaria,
                "alterado": false,
                "confirmado": false,
                "origem": "web",
                "requisicoes": previa.requisicoes,
            });
            crate::indice_local::anexar_avisos(&mut valor, indice);
            println!("{valor}");
        } else if necessaria {
            println!(
                "Prévia de alteração no processo {}: {:?} → {:?}. Nenhuma escrita.",
                numero.as_str(), previa.anterior, especificacao
            );
        } else {
            println!("Processo {}: especificação já igual. Nenhuma escrita.", numero.as_str());
        }
        Ok(())
    }))
}

// A confirmação do alvo usa identidade do formulário e da action, não o texto.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::formulario_web::tests::{instalacao, tela_iso};
    use crate::propriedades_processo::tests_apoio::{
        link_das_propriedades, tela_das_propriedades,
    };
    use crate::tela_html::HtmlServido;

    const NUP: &str = "00001.000001/2026-01";

    fn preparar_html(html: &str) -> Result<PropriedadesLidas, ErroAlterar> {
        let html = HtmlServido::de_teste(html);
        let url = format!(
            "https://sei.exemplo.gov.br/sei/{}",
            link_das_propriedades(AcaoDoSei::ProcedimentoAlterar, "10")
        );
        preparar(&instalacao(), &tela_iso(&url, &html), NUP, "10", "Nova", None)
    }

    #[test]
    fn previa_confere_identidade_e_action() {
        let html = tela_das_propriedades(NUP, "10");
        assert!(preparar_html(&html).is_ok());
        let outra_action = html.replacen("id_procedimento=10&", "id_procedimento=11&", 1);
        assert!(matches!(
            preparar_html(&outra_action),
            Err(ErroAlterar::ActionRecusada(EnderecoRecusado::DeOutroProcesso))
        ));
        let outro = tela_das_propriedades(NUP, "11");
        assert!(matches!(
            preparar_html(&outro),
            Err(ErroAlterar::Propriedades(ErroPropriedades::IdDasPropriedadesDiferenteDoAberto))
        ));
    }

    #[test]
    fn previa_recusa_individual() {
        let html = tela_das_propriedades(NUP, "10").replace(
            "name=\"hdnSinIndividual\" value=\"N\"",
            "name=\"hdnSinIndividual\" value=\"S\"",
        );
        assert!(matches!(
            preparar_html(&html),
            Err(ErroAlterar::Propriedades(ErroPropriedades::CadastroIndividualNaoSuportado))
        ));
    }
}

//! `ies processo buscar TERMO [--pagina N]` (contracts/cli/processo-buscar.md):
//! valida o termo (saída 2 antes de ler a configuração), lê os arquivos
//! privados da sessão e publica a resposta observada da pesquisa Web.
use crate::colheita_do_controle::Colheita;
use crate::indice_local::IndiceLocal;
use crate::pesquisa_web::alvo::EmProcessos;
use crate::pesquisa_web::erro_busca::ErroBuscaDeProcesso;
use crate::pesquisa_web::filtros::FiltrosDaPesquisa;
use crate::pesquisa_web::fluxo::{
    BuscaRealizada, PedidoDeBusca, SessaoDaBusca, buscar_colhendo_filtrado,
};
use crate::pesquisa_web::resposta::RespostaDaPesquisa;
use crate::processo::NumeroProcesso;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, configuracao, relogin_cli, transporte};
use serde_json::{Value, json};

/// Opções de leitura de `ies processo NUP` que não combinam com `buscar`.
fn opcoes_de_leitura_presentes(comando: &Processo) -> bool {
    comando.numero.is_some()
        || comando.sem_documentos
        || comando.sem_marcadores
        || comando.sem_andamentos
        || comando.sem_propriedades
        || comando.andamentos_completos
}

/// O JSON da resposta observada; nunca HTML, links, cookies ou `infra_hash`.
pub(crate) fn json_da_busca(termo: &NumeroProcesso, busca: &BuscaRealizada) -> Value {
    let mut valor = json!({
        "termo": termo.as_str(),
        "unidade": busca.unidade,
        "origem": "web",
        "requisicoes": busca.requisicoes,
    });
    let extra = match &busca.resposta {
        // O 302 só traz o identificador interno: nenhum número é inventado.
        RespostaDaPesquisa::ProcessoUnico { id } => json!({
            "resultado": "processo_unico",
            "processo": {"id": id.as_str()},
        }),
        RespostaDaPesquisa::Lista(lista) => json!({
            "resultado": "lista",
            "total_declarado": lista.total_declarado,
            "faixa_observada": {"inicio": lista.faixa.inicio, "fim": lista.faixa.fim},
            "linhas_ausentes_da_faixa": lista.linhas_ausentes_da_faixa,
            "processos": lista
                .processos
                .iter()
                .map(|p| json!({"numero": p.numero.as_str()}))
                .collect::<Vec<_>>(),
        }),
        // Resposta observada para esta conta/unidade, não inexistência.
        RespostaDaPesquisa::SemResultado => json!({
            "resultado": "sem_resultado",
            "total_declarado": 0,
            "faixa_observada": null,
            "processos": [],
        }),
    };
    if let (Some(objeto), Value::Object(campos)) = (valor.as_object_mut(), extra) {
        objeto.extend(campos);
    }
    valor
}

fn humano(termo: &NumeroProcesso, busca: &BuscaRealizada) -> String {
    let detalhe = match &busca.resposta {
        RespostaDaPesquisa::ProcessoUnico { id } => {
            format!("um processo (id interno {})", id.as_str())
        }
        RespostaDaPesquisa::Lista(lista) => {
            let numeros: Vec<&str> = lista.processos.iter().map(|p| p.numero.as_str()).collect();
            let ausentes = match lista.linhas_ausentes_da_faixa {
                0 => String::new(),
                n => format!(", {n} sem linha na tabela"),
            };
            format!(
                "{} de {} declarado(s), faixa {} a {}{ausentes}: {}",
                numeros.len(),
                lista.total_declarado,
                lista.faixa.inicio,
                lista.faixa.fim,
                numeros.join(", ")
            )
        }
        RespostaDaPesquisa::SemResultado => {
            "nenhum resultado para esta conta e unidade (não prova inexistência)".to_owned()
        }
    };
    format!(
        "IES: pesquisa de {:?} na unidade {}: {detalhe} ({} requisições).",
        termo.as_str(),
        busca.unidade,
        busca.requisicoes
    )
}

/// Termo e datas já conferidos pelo clap; o período e a mistura com as
/// opções de leitura são uso inválido (saída 2) antes da configuração.
fn buscar(
    cli: &Cli,
    comando: &Processo,
    termo: &NumeroProcesso,
    pagina: Option<u32>,
    filtros: FiltrosDaPesquisa<'_>,
) -> anyhow::Result<()> {
    if let (Some(de), Some(ate)) = (filtros.de, filtros.ate) {
        let inicio = format!("{}{}{}", &de[6..10], &de[3..5], &de[0..2]);
        let fim = format!("{}{}{}", &ate[6..10], &ate[3..5], &ate[0..2]);
        if inicio > fim {
            crate::uso_invalido::recusar("--de posterior a --ate");
        }
    }
    if opcoes_de_leitura_presentes(comando) {
        crate::uso_invalido::recusar("opções de leitura de `ies processo` não combinam com buscar");
    }
    let arquivos = configuracao::arquivos_de_acesso()?;
    let pedido = PedidoDeBusca {
        termo: termo.as_str(),
        pagina,
        unidade: cli.unidade.as_deref(),
    };
    // Cada tentativa relê a sessão e o índice: depois do relogin, a nova.
    let (mut busca, nota) = relogin_cli::ler(&arquivos, cli.unidade.as_deref(), || {
        let referencias = transporte::sessao::ReferenciasDaSessao::ler(
            &arquivos.url_controle,
            &arquivos.origem,
            &arquivos.cookie,
        )
        .map_err(ErroBuscaDeProcesso::Transporte)?;
        let sessao = SessaoDaBusca {
            url_do_controle: referencias.url_do_controle(),
            origem: referencias.origem(),
            cookie: referencias.cookie(),
        };
        let indice = IndiceLocal::da_configuracao(&arquivos);
        let colheita = Colheita::das_referencias(&indice, &referencias);
        buscar_colhendo_filtrado::<EmProcessos, _>(
            &sessao,
            &pedido,
            filtros,
            &colheita,
            &mut EnvioHttps,
        )
    })
    .map_err(relogin_cli::subir)?;
    busca.requisicoes = relogin_cli::somar(busca.requisicoes, nota.as_ref());
    if cli.json {
        let mut valor = json_da_busca(termo, &busca);
        relogin_cli::anotar(&mut valor, nota.as_ref());
        println!(
            "{}",
            serde_json::to_string_pretty(&valor).expect("JSON serializável")
        );
    } else {
        relogin_cli::avisar(cli.json, nota.as_ref());
        println!("{}", humano(termo, &busca));
    }
    Ok(())
}

/// `Some` só para `ies processo buscar`.
pub(crate) fn tentar_buscar(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(comando) = &cli.command else {
        return None;
    };
    let Some(ProcessoAcao::Buscar {
        termo,
        pagina,
        tipo,
        assunto,
        de,
        ate,
    }) = &comando.acao
    else {
        return None;
    };
    Some(buscar(
        cli,
        comando,
        termo,
        *pagina,
        FiltrosDaPesquisa {
            tipo: tipo.as_deref(),
            assunto: assunto.as_deref(),
            de: de.as_deref(),
            ate: ate.as_deref(),
        },
    ))
}

#[cfg(test)]
#[path = "processo_buscar_cli/tests.rs"]
mod tests;

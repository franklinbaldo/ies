//! Hipótese sintética da criação de grupos, com estado SQLite por unidade.
use crate::processo_envio::{campo_unico, decodificar_formulario, pares};
use crate::{Estado, Resposta, escapar, parametro_unico, protocolo_sintetico};
use iesei::grupos_envio::{self, Escopo, Grupo, Membro};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::collections::HashSet;
use std::fmt::Write as _;

const HASH: &str = protocolo_sintetico::INFRA_HASH;

pub(crate) fn acrescentar_menu(
    mut resposta: Resposta,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if resposta.estado != Estado::Ok {
        return resposta;
    }
    if let Ok(Some(controle)) = iesei::banco_cassettes::ler(banco, instalacao, cenario) {
        let escopo = Escopo {
            instalacao,
            cenario,
            unidade: controle.unidade.as_str(),
        };
        if matches!(grupos_envio::unidades(banco, escopo), Ok(Some(_))) {
            let link = format!(
                "<a href='controlador.php?acao=grupo_unidade_listar&amp;infra_hash={HASH}'>Grupos de Envio</a>"
            );
            resposta.corpo = resposta
                .corpo
                .replacen("</body>", &format!("{link}</body>"), 1);
        }
    }
    resposta
}

fn lista(grupos: &[Grupo]) -> Resposta {
    let mut html = format!(
        "<html><body><button id='btnNovo' onclick=\"location.href='controlador.php?acao=grupo_unidade_cadastrar&amp;infra_hash={HASH}'\">Novo</button>\
         <form id='frmGrupoUnidadeLista'>"
    );
    if grupos.is_empty() {
        html.push_str("<div id='divInfraAreaTabela'>Nenhum registro encontrado.</div>");
    } else {
        let plural = if grupos.len() == 1 {
            "registro"
        } else {
            "registros"
        };
        write!(
            html,
            "<table><caption>Grupos de Envio ({} {plural}):</caption><tr><th></th><th>Nome</th><th>Descrição</th><th>Ações</th></tr>",
            grupos.len()
        )
        .expect("String");
        for grupo in grupos {
            let id = escapar(&grupo.id);
            write!(
                html,
                "<tr><td><input type='checkbox' value='{id}'></td><td>{}</td><td>{}</td><td><a href='controlador.php?acao=grupo_unidade_consultar&amp;id_grupo_unidade={id}&amp;infra_hash={HASH}'>Consultar</a></td></tr>",
                escapar(&grupo.nome),
                escapar(&grupo.descricao)
            )
            .expect("String");
        }
        html.push_str("</table>");
    }
    html.push_str("</form></body></html>");
    Resposta::html(html)
}

fn cadastro(grupo: Option<(&Grupo, &[Membro])>) -> Resposta {
    let (id, nome, descricao, membros) = grupo.map_or(("", "", "", &[][..]), |(g, m)| {
        (g.id.as_str(), g.nome.as_str(), g.descricao.as_str(), m)
    });
    let mut html = format!(
        "<html><body><form id='frmGrupoCadastro' method='post' action='controlador.php?acao=grupo_unidade_cadastrar&amp;infra_hash={HASH}'>\
         <input type='hidden' name='hdnIdGrupoUnidade' value='{}'>\
         <input type='text' name='txtNome' value='{}'><textarea name='txaDescricao'>{}</textarea>\
         <select name='selUnidades' multiple>",
        escapar(id),
        escapar(nome),
        escapar(descricao)
    );
    for membro in membros {
        write!(
            html,
            "<option value='{}'>{}</option>",
            escapar(&membro.id),
            escapar(&membro.nome)
        )
        .expect("String");
    }
    html.push_str("</select><input type='hidden' name='hdnUnidades' value=''>");
    if grupo.is_none() {
        write!(
            html,
            "<input type='submit' name='sbmCadastrarGrupoUnidade' value='Salvar'><script>new infraAjaxAutoCompletar('hdnIdUnidade','txtUnidade','controlador_ajax.php?acao_ajax=unidade_auto_completar_envio_processo&acao_origem=grupo_unidade_cadastrar&infra_hash={HASH}');</script>"
        )
        .expect("String");
    }
    html.push_str("</form></body></html>");
    Resposta::html(html)
}

struct Cadastro {
    nome: String,
    descricao: String,
    membros: Vec<Membro>,
}

fn ler_cadastro(corpo: &str, oferecidos: &[Membro]) -> Option<Cadastro> {
    let campos = pares(corpo)?;
    if campo_unico(&campos, "hdnIdGrupoUnidade") != Some("")
        || campo_unico(&campos, "sbmCadastrarGrupoUnidade") != Some("Salvar")
    {
        return None;
    }
    let nome = campo_unico(&campos, "txtNome").and_then(decodificar_formulario)?;
    if nome.trim().is_empty() {
        return None;
    }
    let descricao = campo_unico(&campos, "txaDescricao").and_then(decodificar_formulario)?;
    let unidades = campo_unico(&campos, "hdnUnidades").and_then(decodificar_formulario)?;
    let mut membros = Vec::new();
    let mut ids = HashSet::new();
    if !unidades.is_empty() {
        for item in unidades.split('¥') {
            let (id, nome) = item.split_once('±')?;
            if !ids.insert(id) {
                return None;
            }
            let membro = oferecidos.iter().find(|m| m.id == id && m.nome == nome)?;
            membros.push(membro.clone());
        }
    }
    Some(Cadastro {
        nome,
        descricao,
        membros,
    })
}

fn confirmar(banco: &Connection, escopo: Escopo<'_>, corpo: &str) -> Resposta {
    let Ok(Some(oferecidos)) = grupos_envio::unidades(banco, escopo) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let Some(pedido) = ler_cadastro(corpo, &oferecidos) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    match grupos_envio::criar(
        banco,
        escopo,
        &pedido.nome,
        &pedido.descricao,
        &pedido.membros,
    ) {
        Ok(id) => Resposta::redirecionar(format!(
            "controlador.php?acao=grupo_unidade_listar&id_grupo={id}"
        )),
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    let Ok(Some(controle)) = iesei::banco_cassettes::ler(banco, instalacao, cenario) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let escopo = Escopo {
        instalacao,
        cenario,
        unidade: controle.unidade.as_str(),
    };
    match (metodo, parametro_unico(alvo, "acao")) {
        ("GET", Some("grupo_unidade_listar")) => match grupos_envio::listar(banco, escopo) {
            Ok(Some(grupos)) => lista(&grupos),
            _ => Resposta::nova(Estado::DadosInsuficientes),
        },
        ("GET", Some("grupo_unidade_cadastrar")) => match grupos_envio::unidades(banco, escopo) {
            Ok(Some(_)) => cadastro(None),
            _ => Resposta::nova(Estado::DadosInsuficientes),
        },
        ("POST", Some("grupo_unidade_cadastrar")) => confirmar(banco, escopo, corpo),
        ("GET", Some("grupo_unidade_consultar")) => {
            let Some(id) = parametro_unico(alvo, "id_grupo_unidade") else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            match grupos_envio::obter(banco, escopo, id) {
                Ok(Some((grupo, membros))) => cadastro(Some((&grupo, &membros))),
                Ok(None) => Resposta::nova(Estado::NaoEncontrado),
                Err(_) => Resposta::nova(Estado::DadosInsuficientes),
            }
        }
        _ => Resposta::nova(Estado::MetodoNaoPermitido),
    }
}

/// Marcador sintético de origem separa a consulta de grupos da consulta de envio.
pub(crate) fn autocompletar(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    let caminho = alvo.split_once('?').map_or(alvo, |(caminho, _)| caminho);
    if !caminho.ends_with("/controlador_ajax.php") {
        return crate::despachar(metodo, alvo);
    }
    if metodo != "POST" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(campos) = pares(corpo).filter(|c| c.len() == 1) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Some(termo) = campo_unico(&campos, "palavras_pesquisa")
        .and_then(decodificar_formulario)
        .filter(|s| !s.trim().is_empty())
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Ok(Some(controle)) = iesei::banco_cassettes::ler(banco, instalacao, cenario) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let escopo = Escopo {
        instalacao,
        cenario,
        unidade: controle.unidade.as_str(),
    };
    let Ok(Some(oferecidos)) = grupos_envio::unidades(banco, escopo) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let termo = termo.to_lowercase();
    let mut xml = String::from("<?xml version='1.0' encoding='UTF-8'?><itens>");
    for membro in oferecidos {
        if membro.nome.to_lowercase().contains(&termo) {
            write!(
                xml,
                "<item id='{}' descricao='{}'></item>",
                escapar(&membro.id),
                escapar(&membro.nome)
            )
            .expect("String");
        }
    }
    xml.push_str("</itens>");
    Resposta {
        estado: Estado::Ok,
        tipo_conteudo: "application/xml; charset=utf-8",
        corpo: xml,
        localizacao: None,
        corpo_binario: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cadastro_persiste_uma_identidade_e_recusa_membro_invalido() {
        let banco = Connection::open_in_memory().unwrap();
        banco
            .execute_batch(include_str!("../../iesei/migrations/0045_grupos_envio.sql"))
            .unwrap();
        let instalacao = Instalacao::interpretar("https://sei.example.test").unwrap();
        let escopo = Escopo {
            instalacao: &instalacao,
            cenario: "grupo",
            unidade: "A",
        };
        let membro = Membro {
            id: "001".into(),
            nome: "X - Unidade".into(),
        };
        grupos_envio::semear(&banco, escopo, std::slice::from_ref(&membro)).unwrap();
        let corpo = "hdnIdGrupoUnidade=&txtNome=Grupo&txaDescricao=&hdnUnidades=001%C2%B1X+-+Unidade&sbmCadastrarGrupoUnidade=Salvar";
        let recusa = confirmar(&banco, escopo, &corpo.replace("001", "1"));
        assert_eq!(recusa.estado, Estado::RequisicaoInvalida);
        assert!(
            grupos_envio::listar(&banco, escopo)
                .unwrap()
                .unwrap()
                .is_empty()
        );
        let criado = confirmar(&banco, escopo, corpo);
        assert_eq!(criado.estado, Estado::Redirecionamento);
        assert_eq!(
            criado.localizacao.as_deref(),
            Some("controlador.php?acao=grupo_unidade_listar&id_grupo=1")
        );
        let (grupo, membros) = grupos_envio::obter(&banco, escopo, "1").unwrap().unwrap();
        assert_eq!(grupo.nome, "Grupo");
        assert_eq!(membros, vec![membro]);
        assert_eq!(
            grupos_envio::listar(&banco, escopo).unwrap().unwrap().len(),
            1
        );
    }
}

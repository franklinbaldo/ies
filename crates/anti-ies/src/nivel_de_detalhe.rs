//! A tela do Nível de detalhe do anti-IES reflete somente o SQLite da
//! instalação/cenário. A gravação depende de outra fatia, não há POST fictício.
use crate::{Estado, Resposta, escapar, protocolo_sintetico};
use iesei::colunas_detalhe::{self, Coluna, Configuracao};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

fn html(configuracao: &Configuracao, gravado: bool) -> String {
    let hash = protocolo_sintetico::INFRA_HASH;
    let mut pagina = format!(
        "<html><head><title>SEI - Configurar Detalhe</title></head><body>\
         <form id='frmAcessoExternoCancelar' method='post' onsubmit='return OnSubmitForm();' \
         action='controlador.php?acao=procedimento_configurar_detalhe&amp;acao_origem=procedimento_configurar_detalhe&amp;infra_hash={hash}'>\
         <input type='hidden' id='hdnInfraTipoPagina' name='hdnInfraTipoPagina' value='2'>\
         <button type='submit' id='sbmSalvar' name='sbmSalvar' value='Salvar'>Salvar</button>"
    );
    for opcao in &configuracao.opcoes {
        let campo = opcao.coluna.campo();
        let rotulo = escapar(opcao.coluna.rotulo());
        let checked = if opcao.marcada {
            " checked='checked'"
        } else {
            ""
        };
        write!(
            pagina,
            "<div><input type='checkbox' id='{campo}' name='{campo}' class='infraCheckbox'{checked}>\
             <label for='{campo}'>{rotulo}</label></div>"
        )
        .expect("String");
    }
    pagina.push_str("</form>");
    if gravado {
        pagina.push_str("<script>function inicializar(){if ('1'=='1'){window.parent.location='controlador.php?acao=procedimento_controlar';infraFecharJanelaModal();}}</script>");
    }
    pagina.push_str("</body></html>");
    pagina
}

/// Só aceita os controles publicados pela configuração; não infere
/// opções ausentes nem permite duplicar campos no mesmo POST.
fn campos_marcados(corpo: &str, configuracao: &Configuracao) -> Option<Vec<Coluna>> {
    let (mut pagina, mut salvar) = (false, false);
    let mut marcadas = Vec::new();
    for par in corpo.split('&') {
        let (nome, valor) = par.split_once('=')?;
        match (nome, valor) {
            ("hdnInfraTipoPagina", "2") if !pagina => pagina = true,
            ("sbmSalvar", "Salvar") if !salvar => salvar = true,
            _ => {
                let coluna = configuracao
                    .opcoes
                    .iter()
                    .find(|opcao| opcao.coluna.campo() == nome)?
                    .coluna;
                if valor != "on" || marcadas.contains(&coluna) {
                    return None;
                }
                marcadas.push(coluna);
            }
        }
    }
    (pagina && salvar).then_some(marcadas)
}

/// GET projeta o SQLite; POST autorizado altera exatamente as marcas
/// ofertadas e devolve sinal sintético de gravação (ainda exige GET do IES).
pub(crate) fn responder(
    metodo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !matches!(metodo, "GET" | "POST") {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Ok(Some(configuracao)) = colunas_detalhe::ler(banco, instalacao, cenario) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    if metodo == "GET" {
        return Resposta::html(html(&configuracao, false));
    }
    let Some(marcadas) = campos_marcados(corpo, &configuracao) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    match colunas_detalhe::definir(banco, instalacao, cenario, &marcadas) {
        Ok(Some(atualizada)) => Resposta::html(html(&atualizada, true)),
        Ok(None) | Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iesei::colunas_detalhe::{Coluna, Opcao};
    use iesei::observacao::UnidadeExibida;

    #[test]
    fn form_por_estado_e_nao_por_html_fixo() {
        let config = Configuracao {
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            opcoes: vec![
                Opcao {
                    coluna: Coluna::Atribuicao,
                    marcada: true,
                },
                Opcao {
                    coluna: Coluna::Anotacao,
                    marcada: false,
                },
            ],
        };
        let html = html(&config, false);
        assert!(html.contains("name='chkSinAtribuicao' class='infraCheckbox' checked"));
        assert!(html.contains("name='chkSinAnotacao' class='infraCheckbox'>"));
        assert!(html.contains("id='frmAcessoExternoCancelar'"));
        assert!(!html.contains("name='chkSinMarcadores'"));
    }
}

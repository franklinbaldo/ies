//! Testes sintéticos da gravação privada única (#211).
use super::*;
use std::fs;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Barrier};

struct Pasta(PathBuf);

impl Pasta {
    fn nova(nome: &str) -> Self {
        static N: AtomicU32 = AtomicU32::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let caminho = std::env::temp_dir().join(format!(
            "ies-arquivo-privado-{nome}-{}-{n}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&caminho);
        fs::create_dir(&caminho).unwrap();
        // Independe da umask: a pasta do teste só é gravável pelo dono.
        #[cfg(unix)]
        definir_modo(&caminho, 0o700);
        Self(caminho)
    }

    fn join(&self, nome: &str) -> PathBuf {
        self.0.join(nome)
    }

    /// Nomes presentes, ordenados (inclui ocultos e parciais).
    fn nomes(&self) -> Vec<String> {
        let mut nomes: Vec<String> = fs::read_dir(&self.0)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        nomes.sort();
        nomes
    }
}

impl Drop for Pasta {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(unix)]
fn definir_modo(caminho: &Path, modo: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(caminho, fs::Permissions::from_mode(modo)).unwrap();
}

#[cfg(unix)]
fn modo(caminho: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    fs::symlink_metadata(caminho).unwrap().permissions().mode() & 0o777
}

#[test]
fn exclusiva_cria_arquivo_novo_privado_com_o_conteudo_exato() {
    let pasta = Pasta::nova("novo");
    let alvo = pasta.join("saida.json");
    assert_eq!(gravar(&alvo, b"{\"a\":1}", Politica::exclusiva()), Ok(()));
    assert_eq!(fs::read(&alvo).unwrap(), b"{\"a\":1}");
    assert_eq!(pasta.nomes(), ["saida.json"], "nenhum parcial sobra");
    #[cfg(unix)]
    assert_eq!(modo(&alvo), 0o600);
}

#[test]
fn exclusiva_recusa_destino_existente_sem_alterar_e_sem_parcial() {
    let pasta = Pasta::nova("existente");
    let alvo = pasta.join("saida.txt");
    fs::write(&alvo, "ORIGINAL").unwrap();
    assert_eq!(
        gravar(&alvo, b"INTRUSO", Politica::exclusiva()),
        Err(FalhaDaGravacao::DestinoJaExiste)
    );
    assert_eq!(fs::read_to_string(&alvo).unwrap(), "ORIGINAL");
    assert_eq!(pasta.nomes(), ["saida.txt"]);
}

#[test]
fn substituicao_troca_o_conteudo_inteiro_e_fica_privada() {
    let pasta = Pasta::nova("substitui");
    let alvo = pasta.join("cookie");
    assert_eq!(gravar(&alvo, b"A=1\n", Politica::substituicao()), Ok(()));
    assert_eq!(gravar(&alvo, b"B=2\n", Politica::substituicao()), Ok(()));
    assert_eq!(fs::read_to_string(&alvo).unwrap(), "B=2\n");
    assert_eq!(pasta.nomes(), ["cookie"]);
    #[cfg(unix)]
    assert_eq!(modo(&alvo), 0o600);
}

#[cfg(unix)]
#[test]
fn link_simbolico_no_destino_nunca_e_seguido_em_nenhum_modo() {
    let pasta = Pasta::nova("link-destino");
    let original = pasta.join("original");
    fs::write(&original, "NAO ALTERAR").unwrap();
    let alvo = pasta.join("alvo");
    std::os::unix::fs::symlink(&original, &alvo).unwrap();
    assert_eq!(
        gravar(&alvo, b"X", Politica::exclusiva()),
        Err(FalhaDaGravacao::DestinoJaExiste)
    );
    assert_eq!(
        gravar(&alvo, b"X", Politica::substituicao()),
        Err(FalhaDaGravacao::DestinoNaoEhArquivoRegular)
    );
    assert_eq!(fs::read_to_string(&original).unwrap(), "NAO ALTERAR");
    assert!(
        fs::symlink_metadata(&alvo)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(pasta.nomes(), ["alvo", "original"]);
}

/// Um link plantado no nome final (o ataque clássico de quem pode
/// escrever no diretório) nunca é atravessado, nem pendente (apontando
/// para um arquivo que ainda não existe), nem plantado depois de abrir o
/// parcial, em nenhum modo, num diretório `/tmp` (sticky, `0o1777`).
#[cfg(unix)]
#[test]
fn link_plantado_no_nome_final_nunca_e_atravessado_em_nenhum_modo() {
    let pasta = Pasta::nova("link-plantado");
    let comum = pasta.join("comum");
    fs::create_dir(&comum).unwrap();
    definir_modo(&comum, 0o1777);
    let vitima = pasta.join("vitima");
    let alvo = comum.join("sessao");
    std::os::unix::fs::symlink(&vitima, &alvo).unwrap();
    for politica in [Politica::exclusiva(), Politica::substituicao()] {
        assert!(gravar(&alvo, b"SEGREDO", politica).is_err(), "{politica:?}");
        assert!(!vitima.exists(), "o link pendente não criou a vítima");
    }
    fs::remove_file(&alvo).unwrap();
    for politica in [Politica::exclusiva(), Politica::substituicao()] {
        let mut parcial = Parcial::abrir(&alvo, politica).unwrap();
        parcial.escrever(b"SEGREDO").unwrap();
        std::os::unix::fs::symlink(&vitima, &alvo).unwrap();
        assert!(parcial.confirmar().is_err(), "{politica:?}");
        assert!(!vitima.exists());
        assert!(
            fs::symlink_metadata(&alvo)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        fs::remove_file(&alvo).unwrap();
    }
    assert_eq!(fs::read_dir(&comum).unwrap().count(), 0, "nenhum parcial");
}

/// `/home` → `/var/home` (Silverblue) ou `/tmp` → `/private/tmp` (macOS):
/// um ancestral link não impede a gravação, em nenhum modo, nem a
/// criação dos níveis ausentes (`0700`) dentro do diretório real.
#[cfg(unix)]
#[test]
fn ancestral_link_simbolico_e_aceito_e_o_arquivo_vai_para_o_diretorio_real() {
    let pasta = Pasta::nova("ancestral");
    let real = pasta.join("var").join("home").join("pessoa");
    fs::create_dir_all(&real).unwrap();
    for nivel in [pasta.join("var"), pasta.join("var/home"), real.clone()] {
        definir_modo(&nivel, 0o755);
    }
    std::os::unix::fs::symlink(pasta.join("var").join("home"), pasta.join("home")).unwrap();
    let pela_casa = pasta.join("home").join("pessoa");
    assert_eq!(conferir_diretorios(&pela_casa.join("x"), false), Ok(()));
    assert_eq!(
        gravar(&pela_casa.join("caixa.json"), b"{}", Politica::exclusiva()),
        Ok(())
    );
    assert_eq!(
        gravar(&pela_casa.join("cookie"), b"A=1", Politica::substituicao()),
        Ok(())
    );
    assert_eq!(
        gravar(&pela_casa.join("cookie"), b"B=2", Politica::substituicao()),
        Ok(())
    );
    let banco = pela_casa.join(".local").join("ies").join("ies.sqlite3");
    assert_eq!(
        gravar(&banco, b"", Politica::exclusiva().criando_diretorios()),
        Ok(())
    );
    assert_eq!(fs::read(real.join("caixa.json")).unwrap(), b"{}");
    assert_eq!(fs::read(real.join("cookie")).unwrap(), b"B=2");
    assert_eq!(modo(&real.join("cookie")), 0o600);
    assert_eq!(modo(&real.join(".local")), 0o700);
    assert_eq!(modo(&real.join(".local/ies")), 0o700);
    assert!(real.join(".local/ies/ies.sqlite3").is_file());
    let mut nomes: Vec<_> = fs::read_dir(&real)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    nomes.sort();
    assert_eq!(nomes, [".local", "caixa.json", "cookie"], "nenhum parcial");
}

/// O `DIR` que a pessoa deu (`--saida atalho`) pode ser ele mesmo um link:
/// a peça examina e grava no diretório real.
#[cfg(unix)]
#[test]
fn diretorio_dado_como_link_simbolico_e_aceito() {
    let pasta = Pasta::nova("dir-link");
    let real = pasta.join("real");
    fs::create_dir(&real).unwrap();
    definir_modo(&real, 0o700);
    let atalho = pasta.join("atalho");
    std::os::unix::fs::symlink(&real, &atalho).unwrap();
    for (nome, politica) in [
        ("a.txt", Politica::exclusiva()),
        ("b.txt", Politica::substituicao()),
        ("c.txt", Politica::exclusiva().criando_diretorios()),
    ] {
        assert_eq!(gravar(&atalho.join(nome), b"OK", politica), Ok(()));
        assert_eq!(fs::read(real.join(nome)).unwrap(), b"OK");
    }
    assert!(
        fs::symlink_metadata(&atalho)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

/// Diretório real gravável por grupo ou outros, sem sticky bit: recusado
/// antes de criar o parcial, inclusive quando alcançado por um link.
#[cfg(unix)]
#[test]
fn diretorio_gravavel_por_grupo_ou_outros_e_recusado_antes_de_criar() {
    let pasta = Pasta::nova("aberto");
    let aberto = pasta.join("aberto");
    fs::create_dir(&aberto).unwrap();
    let atalho = pasta.join("atalho");
    std::os::unix::fs::symlink(&aberto, &atalho).unwrap();
    for bits in [0o770, 0o730, 0o707, 0o702, 0o777] {
        definir_modo(&aberto, bits);
        for dir in [&aberto, &atalho] {
            for politica in [
                Politica::exclusiva(),
                Politica::substituicao(),
                Politica::exclusiva().criando_diretorios(),
            ] {
                assert_eq!(
                    gravar(&dir.join("x"), b"SEGREDO", politica),
                    Err(FalhaDaGravacao::DiretorioGravavelPorOutros),
                    "{bits:o}"
                );
            }
            assert_eq!(
                conferir_diretorios(&dir.join("x"), false),
                Err(FalhaDaGravacao::DiretorioGravavelPorOutros)
            );
        }
        assert_eq!(fs::read_dir(&aberto).unwrap().count(), 0, "{bits:o}");
    }
    // Só escrita conta: leitura e busca por outros não abrem o diretório.
    definir_modo(&aberto, 0o755);
    assert_eq!(
        gravar(&aberto.join("x"), b"OK", Politica::exclusiva()),
        Ok(())
    );
}

/// `/tmp` (`0o1777`): o sticky bit impede que outro usuário remova ou
/// troque o que é nosso, e a criação exclusiva não segue o que ele plantou.
#[cfg(unix)]
#[test]
fn diretorio_sticky_gravavel_por_todos_e_aceito() {
    let pasta = Pasta::nova("sticky");
    let comum = pasta.join("comum");
    fs::create_dir(&comum).unwrap();
    for bits in [0o1777, 0o1770] {
        definir_modo(&comum, bits);
        let nome = format!("x-{bits:o}");
        assert_eq!(
            gravar(&comum.join(&nome), b"A", Politica::exclusiva()),
            Ok(())
        );
        assert_eq!(
            gravar(&comum.join(&nome), b"B", Politica::substituicao()),
            Ok(())
        );
        assert_eq!(fs::read(comum.join(&nome)).unwrap(), b"B");
        assert_eq!(modo(&comum.join(&nome)), 0o600);
    }
}

#[test]
fn diretorio_ausente_e_falha_tipada_sem_criar_nada() {
    let pasta = Pasta::nova("ausente");
    let destino = pasta.join("nao").join("existe.txt");
    assert_eq!(
        gravar(&destino, b"X", Politica::exclusiva()),
        Err(FalhaDaGravacao::DiretorioAusente)
    );
    let arquivo = pasta.join("arquivo");
    fs::write(&arquivo, "").unwrap();
    assert_eq!(
        gravar(&arquivo.join("x"), b"X", Politica::exclusiva()),
        Err(FalhaDaGravacao::DiretorioAusente)
    );
    assert_eq!(pasta.nomes(), ["arquivo"]);
}

#[test]
fn criando_diretorios_cada_nivel_novo_nasce_0700() {
    let pasta = Pasta::nova("cria-dir");
    let destino = pasta.join("a").join("b").join("banco");
    let politica = Politica::exclusiva().criando_diretorios();
    assert_eq!(gravar(&destino, b"", politica), Ok(()));
    assert_eq!(fs::read(&destino).unwrap(), b"");
    #[cfg(unix)]
    {
        assert_eq!(modo(&pasta.join("a")), 0o700);
        assert_eq!(modo(&pasta.join("a").join("b")), 0o700);
        assert_eq!(modo(&destino), 0o600);
    }
}

#[test]
fn sem_nome_de_arquivo_e_recusado() {
    assert_eq!(
        gravar(Path::new("/"), b"X", Politica::exclusiva()),
        Err(FalhaDaGravacao::SemNomeDeArquivo)
    );
    let pasta = Pasta::nova("sem-nome");
    assert_eq!(
        gravar(&pasta.join(".."), b"X", Politica::substituicao()),
        Err(FalhaDaGravacao::SemNomeDeArquivo)
    );
}

#[test]
fn acima_do_limite_falha_sem_destino_nem_parcial() {
    let pasta = Pasta::nova("limite");
    let alvo = pasta.join("grande");
    let politica = Politica::exclusiva().com_limite(4);
    assert_eq!(
        gravar(&alvo, b"12345", politica),
        Err(FalhaDaGravacao::AcimaDoLimite)
    );
    assert_eq!(gravar(&alvo, b"1234", politica), Ok(()));
    let mut parcial = Parcial::abrir(&pasta.join("fluxo"), politica).unwrap();
    assert_eq!(parcial.escrever(b"12"), Ok(()));
    assert_eq!(
        parcial.escrever(b"345"),
        Err(FalhaDaGravacao::AcimaDoLimite)
    );
    drop(parcial);
    assert_eq!(pasta.nomes(), ["grande"]);
}

#[test]
fn falha_no_meio_da_escrita_nao_deixa_parcial_nem_destino() {
    let pasta = Pasta::nova("meio");
    let alvo = pasta.join("anexo.pdf");
    let mut parcial = Parcial::abrir(&alvo, Politica::exclusiva()).unwrap();
    parcial.escrever(b"%PDF-1.7 metade").unwrap();
    let temporario = parcial.temporario().to_owned();
    assert!(temporario.exists(), "o parcial existe enquanto escreve");
    assert!(!alvo.exists(), "o nome final só aparece na confirmação");
    // O download falhou: o chamador desiste sem confirmar.
    drop(parcial);
    assert!(!temporario.exists());
    assert_eq!(pasta.nomes(), Vec::<String>::new());
}

#[test]
fn parcial_confirmado_aparece_inteiro_no_nome_final() {
    let pasta = Pasta::nova("confirma");
    let alvo = pasta.join("anexo.pdf");
    let mut parcial = Parcial::abrir(&alvo, Politica::exclusiva()).unwrap();
    parcial.escrever(b"%PDF").unwrap();
    parcial.escrever(b"-1.7").unwrap();
    parcial.sincronizar().unwrap();
    assert_eq!(parcial.confirmar(), Ok(()));
    assert_eq!(fs::read(&alvo).unwrap(), b"%PDF-1.7");
    assert_eq!(pasta.nomes(), ["anexo.pdf"]);
}

#[test]
fn destino_que_surge_antes_da_confirmacao_vence_e_nada_e_sobrescrito() {
    let pasta = Pasta::nova("surge");
    let alvo = pasta.join("foto.html");
    let mut parcial = Parcial::abrir(&alvo, Politica::exclusiva()).unwrap();
    parcial.escrever(b"MEU").unwrap();
    fs::write(&alvo, "DO OUTRO").unwrap();
    assert_eq!(parcial.confirmar(), Err(FalhaDaGravacao::DestinoJaExiste));
    assert_eq!(fs::read_to_string(&alvo).unwrap(), "DO OUTRO");
    assert_eq!(pasta.nomes(), ["foto.html"]);
}

#[test]
fn dois_criadores_exclusivos_so_um_vence() {
    let pasta = Pasta::nova("corrida");
    let alvo = Arc::new(pasta.join("caixa.json"));
    let criadores = 8;
    let largada = Arc::new(Barrier::new(criadores));
    let tarefas: Vec<_> = (0..criadores)
        .map(|i| {
            let (alvo, largada) = (Arc::clone(&alvo), Arc::clone(&largada));
            std::thread::spawn(move || {
                let conteudo = format!("criador-{i}");
                largada.wait();
                (
                    conteudo.clone(),
                    gravar(&alvo, conteudo.as_bytes(), Politica::exclusiva()),
                )
            })
        })
        .collect();
    let resultados: Vec<_> = tarefas.into_iter().map(|t| t.join().unwrap()).collect();
    let vencedores: Vec<_> = resultados.iter().filter(|(_, r)| r.is_ok()).collect();
    assert_eq!(vencedores.len(), 1, "{resultados:?}");
    for (_, resultado) in &resultados {
        assert!(matches!(
            resultado,
            Ok(()) | Err(FalhaDaGravacao::DestinoJaExiste)
        ));
    }
    assert_eq!(fs::read_to_string(&*alvo).unwrap(), vencedores[0].0);
    assert_eq!(pasta.nomes(), ["caixa.json"], "nenhum parcial sobra");
}

#[test]
fn parcial_de_processo_interrompido_e_varrido_so_quando_antigo() {
    let pasta = Pasta::nova("varre");
    let antigo = pasta.join(".DOC.v1.html.parcial-4194304");
    let recente = pasta.join(".DOC.v1.html.parcial-4194305-0");
    let alheio = pasta.join(".outro.html.parcial-4194304");
    for caminho in [&antigo, &recente, &alheio] {
        fs::write(caminho, "metade").unwrap();
    }
    let ha_duas_horas = std::time::SystemTime::now() - std::time::Duration::from_secs(7_200);
    for caminho in [&antigo, &alheio] {
        fs::File::options()
            .write(true)
            .open(caminho)
            .unwrap()
            .set_modified(ha_duas_horas)
            .unwrap();
    }
    assert_eq!(
        gravar(&pasta.join("DOC.v1.html"), b"<html>", Politica::exclusiva()),
        Ok(())
    );
    assert_eq!(
        pasta.nomes(),
        [
            ".DOC.v1.html.parcial-4194305-0",
            ".outro.html.parcial-4194304",
            "DOC.v1.html"
        ]
    );
}

/// `create_new(true)` só nesta peça e em testes: nenhuma gravação privada
/// paralela com política própria.
#[test]
fn nenhuma_outra_gravacao_privada_usa_create_new() {
    fn visitar(pasta: &Path, achados: &mut Vec<String>) {
        for entrada in fs::read_dir(pasta).unwrap() {
            let caminho = entrada.unwrap().path();
            if caminho.is_dir() {
                visitar(&caminho, achados);
                continue;
            }
            let nome = caminho.to_string_lossy().replace('\\', "/");
            let de_teste = nome.contains("/tests") || nome.contains("tests_");
            if caminho.extension().is_some_and(|ext| ext == "rs")
                && !de_teste
                && !nome.ends_with("src/arquivo_privado.rs")
                && fs::read_to_string(&caminho)
                    .unwrap()
                    .contains(concat!("create_new", "(true)"))
            {
                achados.push(nome);
            }
        }
    }
    let mut achados = Vec::new();
    visitar(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut achados,
    );
    assert!(achados.is_empty(), "{achados:?}");
}

//! O diretório do destino (05/11/2026): links no caminho aceitos, modo
//! conferido só no diretório do IES, nome final nunca seguido. Só Unix.
use super::*;

/// Um link plantado no nome final (o ataque clássico de quem pode
/// escrever no diretório) nunca é atravessado, nem pendente (apontando
/// para um arquivo que ainda não existe), nem plantado depois de abrir o
/// parcial, em nenhum modo, num diretório `/tmp` (sticky, `0o1777`).
#[test]
fn link_plantado_no_nome_final_nunca_e_atravessado_em_nenhum_modo() {
    let pasta = Pasta::nova("link-plantado");
    let comum = pasta.join("comum");
    fs::create_dir(&comum).unwrap();
    definir_modo(&comum, 0o1777);
    let vitima = pasta.join("vitima");
    let alvo = comum.join("sessao");
    std::os::unix::fs::symlink(&vitima, &alvo).unwrap();
    for politica in todas_as_politicas() {
        assert!(gravar(&alvo, b"SEGREDO", politica).is_err(), "{politica:?}");
        assert!(!vitima.exists(), "o link pendente não criou a vítima");
    }
    fs::remove_file(&alvo).unwrap();
    for politica in todas_as_politicas() {
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
    assert_eq!(conferir_diretorios(&pela_casa.join("x"), DO_IES), Ok(()));
    assert_eq!(
        gravar(
            &pela_casa.join("caixa.json"),
            b"{}",
            Politica::exclusiva().diretorio_do_ies()
        ),
        Ok(())
    );
    assert_eq!(
        gravar(
            &pela_casa.join("cookie"),
            b"A=1",
            Politica::substituicao().diretorio_do_ies()
        ),
        Ok(())
    );
    assert_eq!(
        gravar(
            &pela_casa.join("cookie"),
            b"B=2",
            Politica::substituicao().diretorio_do_ies()
        ),
        Ok(())
    );
    let banco = pela_casa.join(".local").join("ies").join("ies.sqlite3");
    assert_eq!(
        gravar(
            &banco,
            b"",
            Politica::exclusiva()
                .diretorio_do_ies()
                .criando_diretorios()
        ),
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
#[test]
fn diretorio_dado_como_link_simbolico_e_aceito() {
    let pasta = Pasta::nova("dir-link");
    let real = pasta.join("real");
    fs::create_dir(&real).unwrap();
    definir_modo(&real, 0o700);
    let atalho = pasta.join("atalho");
    std::os::unix::fs::symlink(&real, &atalho).unwrap();
    for (nome, politica) in [
        ("a.txt", Politica::exclusiva().diretorio_do_ies()),
        ("b.txt", Politica::substituicao().diretorio_do_ies()),
        (
            "c.txt",
            Politica::exclusiva()
                .diretorio_do_ies()
                .criando_diretorios(),
        ),
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

/// Diretório do IES (sessão, índice, configuração): o diretório real
/// gravável por grupo ou outros, sem sticky bit, é recusado antes de
/// criar o parcial, inclusive quando alcançado por um link.
#[test]
fn diretorio_do_ies_gravavel_por_grupo_ou_outros_e_recusado_antes_de_criar() {
    let pasta = Pasta::nova("aberto");
    let aberto = pasta.join("aberto");
    fs::create_dir(&aberto).unwrap();
    let atalho = pasta.join("atalho");
    std::os::unix::fs::symlink(&aberto, &atalho).unwrap();
    for bits in [0o770, 0o730, 0o707, 0o702, 0o777] {
        definir_modo(&aberto, bits);
        for dir in [&aberto, &atalho] {
            for politica in [
                DO_IES,
                Politica::substituicao().diretorio_do_ies(),
                DO_IES.criando_diretorios(),
            ] {
                assert_eq!(
                    gravar(&dir.join("x"), b"SEGREDO", politica),
                    Err(FalhaDaGravacao::DiretorioGravavelPorOutros),
                    "{bits:o}"
                );
            }
            assert_eq!(
                conferir_diretorios(&dir.join("x"), DO_IES),
                Err(FalhaDaGravacao::DiretorioGravavelPorOutros)
            );
        }
        assert_eq!(fs::read_dir(&aberto).unwrap().count(), 0, "{bits:o}");
    }
    // Só escrita conta: leitura e busca por outros não abrem o diretório.
    definir_modo(&aberto, 0o755);
    assert_eq!(gravar(&aberto.join("x"), b"OK", DO_IES), Ok(()));
}

/// Diretório da pessoa (`--saida DIR`): o modo é responsabilidade dela.
/// Gravável pelo grupo (`0775`) ou por todos (`0777`) é aceito; o link
/// plantado no nome final continua nunca atravessado.
#[test]
fn diretorio_do_usuario_gravavel_por_grupo_ou_outros_e_aceito() {
    let pasta = Pasta::nova("usuario");
    let saida = pasta.join("saida");
    fs::create_dir(&saida).unwrap();
    let vitima = pasta.join("vitima");
    for bits in [0o775, 0o777] {
        definir_modo(&saida, bits);
        let nome = format!("doc-{bits:o}.txt");
        assert_eq!(gravar(&saida.join(&nome), b"A", DO_USUARIO), Ok(()));
        assert_eq!(
            gravar(
                &saida.join(&nome),
                b"B",
                Politica::substituicao().diretorio_do_usuario()
            ),
            Ok(())
        );
        assert_eq!(fs::read(saida.join(&nome)).unwrap(), b"B");
        assert_eq!(modo(&saida.join(&nome)), 0o600);
        assert_eq!(conferir_diretorios(&saida.join("x"), DO_USUARIO), Ok(()));
        let plantado = saida.join(format!("plantado-{bits:o}"));
        std::os::unix::fs::symlink(&vitima, &plantado).unwrap();
        for politica in [DO_USUARIO, Politica::substituicao().diretorio_do_usuario()] {
            assert!(gravar(&plantado, b"SEGREDO", politica).is_err());
            assert!(!vitima.exists());
        }
    }
}

/// `/tmp` (`0o1777`): o sticky bit impede que outro usuário remova ou
/// troque o que é nosso, e a criação exclusiva não segue o que ele plantou.
#[test]
fn diretorio_do_ies_sticky_gravavel_por_todos_e_aceito() {
    let pasta = Pasta::nova("sticky");
    let comum = pasta.join("comum");
    fs::create_dir(&comum).unwrap();
    for bits in [0o1777, 0o1770] {
        definir_modo(&comum, bits);
        let nome = format!("x-{bits:o}");
        assert_eq!(
            gravar(
                &comum.join(&nome),
                b"A",
                Politica::exclusiva().diretorio_do_ies()
            ),
            Ok(())
        );
        assert_eq!(
            gravar(
                &comum.join(&nome),
                b"B",
                Politica::substituicao().diretorio_do_ies()
            ),
            Ok(())
        );
        assert_eq!(fs::read(comum.join(&nome)).unwrap(), b"B");
        assert_eq!(modo(&comum.join(&nome)), 0o600);
    }
}

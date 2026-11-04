---
type: SEIObservation
title: Ensaio live de documento assinar (prévia na PEN1)
description: Prévia somente leitura de ies documento assinar na PEN1 5.0.4 (PASS, documento já assinado pela sessão e documento sem a ação na barra), escrita NOT_RUN por aguardar a autorização do proprietário, e o plano da escrita, 04/11/2026
---

# Ensaio live de `documento assinar` — 04/11/2026

Contrato em [documento assinar](../../contracts/cli/documento-assinar.md).
Binário de debug do ramo `slice-assinar` (sobre `7a8b332`, alterações
ainda não commitadas na hora do ensaio), sessão própria da PEN1 5.0.4 de
treinamento (conta pública do órgão 1, unidade TESTE), índice da sessão.
**Nenhum POST de assinatura**: a escrita não se desfaz e exige a
autorização do proprietário.

## Comandos (21:43–21:46 UTC)

| Hora | Comando | Saída | Requisições | Resultado |
| --- | --- | --- | --- | --- |
| 21:43:50 | `documento --processo 08227.000048/2026-12 assinar 0009007` | 1 | — | `sessao_encerrada_rode_ies_login` (sessão gravada vencida) |
| 21:44:20 | `login` | 0 | 7 | sessão nova |
| 21:45:14 | `documento --processo 08227.000048/2026-12 assinar 0009007` | 0 | 4 | **PASS** |
| 21:46:00 | `documento --processo 08227.000048/2026-12 assinar 0009006` | 1 | — | **PASS** (`assinatura_nao_publicada_na_barra`) |

Intervalo entre comandos de 27 s (o login, um a menos que o combinado),
36 s e 43 s; nenhum retry; nenhuma saída trouxe URL, `infra_hash`, cookie
ou senha (a senha nem foi lida: só `--confirmar` a lê).

**0009007** (Despacho do processo de teste, já assinado antes pela mesma
conta): abertura de 2 GET pelo link de trabalho guardado, a tela do
documento e a tela de assinatura (os dois GETs da prévia, sem efeito
observado). Saiu `ja_assinado_pela_sessao: true`, `alterado: false`, o
assinante da tela (`Usuário Teste Orgão 1`) igual ao nome da caneta, a
caneta `Assessor(a)`/`TESTE`, 21 cargos e o sugerido (`Agente Fiscalizador
de Contrato`). Confirma ao vivo, no 5.0.4: a ação da barra, a caneta, o
a3 da tela do documento, o link de assinar e a tela `frmAssinaturas`
conferida com a lista fechada de controles. `edicao: nao_publicada`: a
barra deste documento não publica `editarConteudo` (informativo).

**0009006** (Solicitação do mesmo processo): a barra não oferece assinar;
recusa depois da abertura, sem GET das telas.

## Escrita: NOT_RUN

Bloqueio: a assinatura **não se desfaz** (nem pela CLI nem pela tela; editar
o conteúdo depois só a invalida) e o proprietário ainda não a autorizou. Nenhum
replay substitui o ensaio da escrita.

## Plano da escrita para o proprietário

1. **Instalação**: PEN1 5.0.4 (a mesma sessão; forma `var
   linkAssinarDocumento` do 5.0, caneta com `assinatura_listar`), depois a
   UnB 4.0.9 (forma da função de três linhas), como pede a #174.
2. **Documento**: um Despacho **criado para o ensaio**, sem assinatura, num
   processo de teste da unidade da sessão (por exemplo, incluído à mão em
   `08227.000048/2026-12`), com o texto "ensaio IES documento assinar
   2026-11-04". Nunca um documento de outro ensaio.
3. **Prévia** (sem POST): `ies documento --processo NUP assinar DOC
   --json`; PASS se `ja_assinado_pela_sessao: false`, `assinaturas: []`,
   o cargo pretendido em `cargos` e `requisicoes` = abertura + 2.
4. **Escrita** (≥ 30 s depois): `ies documento --processo NUP assinar DOC
   --cargo "Assessor(a)" --confirmar --json`, com `IES_CREDENCIAIS_FILE`
   apontando o arquivo `0600` da conta. Orçamento: abertura + 2 GET + 1 POST
   + 1 GET de verificação (com a árvore guardada pela abertura: 2 + 2 + 1 +
   1 = 6). Um comando, nunca repetido.
5. **PASS** se: saída 0, `alterado: true`, `assinaturas_depois` com um
   único item igual ao assinante da tela e ao cargo pedido,
   `reconciliado_por_leitura: false` e `requisicoes` no orçamento; e uma
   leitura independente (≥ 30 s depois, `ies documento DOC`, ou a árvore
   na tela) mostra a caneta. Saída 3: **não repetir**; conferir a árvore à
   mão e registrar. Saída 1 `assinatura_nao_aplicada`: registrar a
   `falha_do_envio` (ex. senha recusada) e parar.
6. Registrar aqui hora, requisições, saída e o estado da árvore depois.

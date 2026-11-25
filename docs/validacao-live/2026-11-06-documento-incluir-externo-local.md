---
type: SEIObservation
title: Pré-inspeção local de PDF externo — issue 332
description: Evidências test-first da inspeção offline sem sessão, upload, escrita ou ensaio de produção
---

# #332 — inspeção local de PDF antes do envio

## Origem do comportamento

O arquivo `franklinbaldo/todos/src/todos/documento_externo.py` documenta observação prévia em **UnB TREINAMENTO, SEI 4**, em 11/10/2026: cadastro `documento_receber`, upload temporário `documento_upload_anexo`, recibo, `hdnAnexos` e confirmação remota por releitura. Isto é **oráculo anterior do Todos**, não ensaio desta fatia nem confirmação da instalação atual.

## Testes sintéticos locais

- **RED:** [GitHub Actions 42330589633](https://github.com/franklinbaldo/ies/actions/runs/42330589633), commit `2474a7a`, **6 FAIL novos, 1.782 PASS, 22 ignored preexistentes**; os testes foram publicados com `inspecionar_pdf` ainda propositalmente retornando `Ausente`. O mesmo run registrou Rustfmt/Clippy/subcódigos falhando pelo esqueleto, não tratá-lo como GREEN.
- **GREEN:** implementação publicada posteriormente com inspeção offline tipada, SHA-256 em blocos e recusa de arquivo não regular, symlink, extensão, assinatura, vazio e limite observado. **CI final deve ser conferido no SHA exato da PR #352**; não atribuir PASS antes do gate.
- **Orçamento:** todos os testes novos operam `std::fs` com arquivos sintéticos e não acessam SEI. A CLI não abre sessão nem rede.

## RED adicional — validação local do nome

Em 07/11/2026 foram acrescentados três cenários: nomes contendo `&#` (incompatíveis com o upload observado no Todos), sequência de escape de terminal no nome em Unix e rejeição pela CLI antes da sessão. O RED [run 42334803924](https://github.com/franklinbaldo/ies/actions/runs/42334803924), head `d558491`, registrou no teste unitário **1.805 PASS, 2 FAIL, 23 ignored**, com `Rustfmt` e `Clippy` aprovados. O teste CLI fica para o estágio de integração após a correção. GREEN deve ser comprovado em run posterior do head final; nenhum teste RED é considerado PASS antecipadamente.

## Retificação do contrato de erros — ADR 0005 (07/11/2026)

A fatia anterior acrescentara nomes textuais de subcódigos, embora novas
fatias devam publicar o erro tipado por Serde. Correção na mesma PR #352:
`FalhaPdf::subcodigo()` removida; CLI publica `causa` diretamente,
sem tabela paralela. Os testes de identidade voltaram a comparar o erro.

- **RED:** [CI 42335719860](https://github.com/franklinbaldo/ies/actions/runs/42335719860),
  head `c5c78fc`: 5 falhas novas no teste de CLI, 1 teste de CLI
  aprovado; 1.807 testes unitários aprovados, 23 ignorados já existentes.
  O hook Rustfmt também acusou a formatação da etapa RED.
- **GREEN:** candidato no commit `07a89d8` ([CI 42335957617](https://github.com/franklinbaldo/ies/actions/runs/42335957617)).
  Consultar o resultado do SHA final após ajustes documentais; esta linha
  não presume sucesso antes de todos os gates.
- **Live remoto:** NOT_RUN; nenhuma sessão, GET, POST, upload, escrita ou
  resíduo SEI. A confirmação continua recusada antes da rede.

## RED → GREEN adicional — integridade da apresentação (07/11/2026)

- **RED publicado antes da correção:** commit `c76ac72`, [run CI 42339576715](https://github.com/franklinbaldo/ies/actions/runs/42339576715). Suite Rust: **1.810 PASS, 1 FAIL novo, 23 ignored**. O teste unitário demonstrou que `char::is_control` aceitava um nome com U+202E e devolvia `PdfLocal`, em vez de `FalhaPdf::NomeInvalido`. O teste CLI para tipo com U+202E foi publicado no mesmo RED, mas o hook não o executou após a falha unitária.
- **GREEN em código:** usar o mesmo predicado de apresentação para nomes de PDF e texto do tipo solicitado; rejeitar controles e marcadores bidirecionais Unicode (U+061C, U+200E/F, U+202A–E e U+2066–9). São **caracteres que reordenam texto na saída humana**, não um limite de charset alegado pelo SEI. Os erros existentes `Pdf(NomeInvalido)` e `Tipo(Controle)` permanecem tipados; sem novo subcódigo. `--confirmar` continua recusado antes da sessão/rede.
- **CI intermediário:** [42339719537](https://github.com/franklinbaldo/ies/actions/runs/42339719537) verificou Rust/Clippy/OKF com PASS, mas falhou somente em Rustfmt de três trechos; correções de formatação foram commitadas antes do head final. **Consultar o CI do head atual** para atestar GREEN definitivo.
- **Acervo adicional lido, sem rede SEI:** [cassette PEN3 de prévia](../../cassettes/pen3/documento-juntar-previa-pen3--ec754357a3c7.md) (SEI 5.0.4: 9 interações na captura, 5 GET + 4 POST), [testes UnB 4](https://github.com/franklinbaldo/todos/blob/main/tests/test_documento_juntar_010.py) e [SEI 5](https://github.com/franklinbaldo/todos/blob/main/tests/test_documento_juntar_sei5_010.py). A evidência de versões diferentes, recibo e duas mutações está detalhada no [cartão da operação](../../contracts/evidencias/documento-incluir-externo.md). Contagens dessas capturas **não** são requisições desta execução do IES.

## Teste adicional — mutação concorrente do PDF (07/11/2026)

- **RED real:** [CI 42343893669](https://github.com/franklinbaldo/ies/actions/runs/42343893669), head `10180c4`, suíte Rust com **1.811 PASS, 1 FAIL novo, 23 ignored**. O teste altera bytes preservando o mesmo tamanho entre a primeira metadata e a leitura: a implementação anterior retornou `Ok(PdfLocal)` com hash do arquivo modificado, quando o teste exigia `AlteradoDuranteLeitura`. Rustfmt/Clippy/OKF não foram usados como substitutos do RED.
- **GREEN proposto:** comparar metadados do **mesmo descritor** depois de ler até EOF (tamanho, modificação e ctime no Unix) e recusar mudança detectada com um erro tipado. Mesmo conteúdo estável após a inspeção não é garantido para futuro upload; trocar um ancestral por symlink ainda requer proteção específica. CI e wheel precisam ser verificados no SHA de GREEN.
- **Execução remota:** NOT_RUN; **0 GET, 0 POST, 0 upload, 0 resíduos**; teste apenas sintético e local.

## PR #380 — núcleo de recibo temporário sem rede (07/11/2026)

- **RED test-first:** [CI 42369511617](https://github.com/franklinbaldo/ies/actions/runs/42369511617), SHA `42e8088`, **1.872 PASS, 2 FAIL novos, 24 ignored**. Falharam exatamente o aceite do recibo temporário UnB e a recusa explícita `ERRO#motivo`; outras negativas já passaram no stub fechado.
- **GREEN:** implementado um interpretador tipado de seis campos `#`, sem regex, JS, GET ou POST. Recusa específica `ERRO#motivo`; ausência/ambiguidade de recibo ou HTTP inesperado permanecem efeito incerto sem repetir qualquer escrita. Conferir o gate do **head atual** antes de atestar sua aprovação.
- **Escopo:** testes exclusivamente sintéticos, originados dos cenários UnB do Todos; nenhum upload, cadastro remoto ou releitura do IES foi executado. `--confirmar` segue recusado antes da sessão e da rede. Live: **NOT_RUN**, zero GET, zero POST, zero uploads, zero retries e zero resíduos.

## RED → GREEN do corpo multipart PDF — continuação #380 (07/11/2026)

- **RED compilável:** [CI 42373451642](https://github.com/franklinbaldo/ies/actions/runs/42373451642), SHA `c2d7ed1`: **1.951 testes PASS, 4 FAIL novos, 25 ignored**. Os quatro testes eram: corpo e tamanho exatos, charset da tela, recusa de nome não codificável e fronteira/overflow/escape do nome. O Clippy também sinalizou documentação sem crases no stub RED. O primeiro push de testes [42373341865](https://github.com/franklinbaldo/ies/actions/runs/42373341865) falhou ao compilar por lifetime do `Read`; não é contabilizado como RED de teste.
- **GREEN proposto:** [commit `8e6b876`](https://github.com/franklinbaldo/ies/commit/8e6b87697be3930e4508d2cb88029114d06dff4a) constrói peças multipart conforme o caso Todos/UnB, compõe `Content-Length` e permite leitura encadeada sem carregar o PDF em memória. Verificar CI do SHA final, inclusive `prek`, antes de declarar todos os gates verdes.
- **Live:** nenhum POST, nenhum GET, nenhum upload. A etapa só gerou bytes sintéticos offline; `--confirmar` não habilitado. A fonte do cenário é o teste do Todos, **não** ensaio do IES em treinamento.

## Transporte multipart e recibo HTTP — bancada sintética (07/11/2026)

- **Dado:** PDF sintético e URL `.invalid` representando o upload temporário da UnB SEI 4. **Quando:** o cliente HTTPS único transmite o multipart pelo proxy falso de testes, com TLS substituído por conexão local. **Então:** são conferidos `filArquivo`, bytes exatos, `Content-Type`, `Content-Length`, um POST sem retry e interpretação do recibo temporário da resposta HTTP. Esse recibo **não** significa inclusão definitiva do documento.
- **Test-first:** houve implementação de stubs RED em commits `3777526` e `c8e3d35` seguida de testes da fronteira Web e recibo; os runs RED [42378193926](https://github.com/franklinbaldo/ies/actions/runs/42378193926) e [42378603800](https://github.com/franklinbaldo/ies/actions/runs/42378603800) foram **cancelados por pushes seguintes**. Não há contagem de FAIL observada nesses runs, portanto não declarar RED comprovado nesta nova fatia. O RED multipart offline anterior teve 4 FAIL comprovadas em [42373451642](https://github.com/franklinbaldo/ies/actions/runs/42373451642).
- **GREEN parcial observado:** [CI 42378278609](https://github.com/franklinbaldo/ies/actions/runs/42378278609) passou suíte Rust, Clippy, OKF e limite de arquivo, mas falhou no Rustfmt (formatação do transporte e teste); correções foram commitadas depois. **Somente CI do SHA final confirma GREEN integral.**
- **Ensaio real do IES no SEI:** **NOT_RUN**; 0 GET, 0 POST, 0 uploads, 0 retries, 0 resíduos. Nenhuma conexão com treinamento ou produção. `ies documento incluir-externo --confirmar` segue fechado antes da sessão/rede. Próximos passos: derivar do formulário servido em treinamento o destino e campos, conectar prévia da CLI, depois upload → cadastro → releitura independente.

## Ensaio ao vivo desta fatia

| Etapa | Resultado | Requisições reais | Resíduo |
| --- | --- | ---: | --- |
| Login | NOT_RUN | 0 | nenhum |
| Prévia remota do cadastro | NOT_RUN | 0 | nenhum |
| Upload temporário | NOT_RUN | 0 | nenhum |
| Cadastro definitivo | NOT_RUN | 0 | nenhum |
| Releitura independente | NOT_RUN | 0 | nenhum |

**Total real nesta fatia: 0 requisições ao SEI, 0 GET, 0 POST, 0 escrita e nenhum resíduo.** Não houve acesso à produção nem a treinamento. O comando `--confirmar` permanece deliberadamente recusado antes de abrir a sessão ou a rede.

## Próxima etapa da issue

Observar cadastro e limites da tela nas instalações de treinamento; modelar as duas etapas mutadoras e a verificação efetiva do anexo sem confiar apenas na árvore; fazer RED de transporte, GREEN, CI, prévia viva imediatamente antes de **um comando confirmado**, releitura e registro do resíduo, sem retry. Não declarar a #332 concluída.

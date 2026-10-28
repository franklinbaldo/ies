---
type: Guide
title: Opções de formulários como conceitos de domínio
description: Tipos, políticas e parâmetros com efeitos administrativos no SEI
---

# Opções dos formulários: o que merece modelagem

Não deduzir uma entidade de cada `select` ou `checkbox`. O critério
é o significado e o efeito: opção que altera permissões, transições
de estado, relacionamentos ou obrigações deve aparecer no contrato
do domínio, ainda que como enum/valor e não entidade persistida.

## Classificação inicial

| Opção observada/documentada | Modelagem recomendada | Motivo |
| --- | --- | --- |
| Nível de acesso: público, restrito, sigiloso | `NivelAcesso` (enum/valor) | Política de visibilidade e credenciais |
| Hipótese legal da restrição | Referência a `HipoteseLegal` | Justificativa obrigatória para restrito/sigiloso |
| Manter processo aberto ao enviar | `ManterAbertoNaOrigem` (bool no comando) | Altera efeito do envio na remetente |
| Retorno programado | `RetornoProgramado` (conceito próprio) | Prazo, unidade demandante, destinatária e ciclo de alteração/retorno |
| Cargo/função ao assinar | `CargoFuncaoAssinatura` contextual à unidade | Opções cadastradas para a unidade, não cargo global do usuário |
| Modalidade de assinatura | `ModalidadeAssinatura` (enum extensível) | Senha/certificado, conforme suporte e autorização |
| Nível de acesso permitido/sugerido do tipo | `PoliticaTipoProcesso` | Validação não é igual a default sugerido |
| Restrições do tipo por órgão/unidade | Relações de elegibilidade | Catálogo disponível depende do contexto |
| Processo único por interessado e tipo | Regra do tipo de processo | Invariante de criação sujeito à configuração |
| Documento interno ou externo | `NaturezaDocumento` | Fluxos e capacidades diferentes |
| Observações desta unidade | `ObservacaoUnidade` contextual | Texto de uma unidade não é editável por outras |

## Três categorias de decisão

- **Valor fechado**: enum pequeno, p.ex. `NivelAcesso`, mantendo
  falha explícita para valor desconhecido e sem converter para falso.
- **Entidade ou vínculo**: `RetornoProgramado`, `HipoteseLegal`,
  `CargoFuncaoAssinatura` (configuração vinculada à unidade).
- **Parâmetro de operação**: `manter_aberto` é uma escolha no envio,
  não uma nova entidade nem estado global do processo.

## Regras importantes da documentação

- Ao enviar, a opção **manter aberto** determina se a unidade
  remetente continua com atuação aberta.
- Retorno programado pode gerar **vários prazos para um processo**.
  O vencimento não retorna automaticamente o processo; concluir
  não encerra a contagem. A unidade demandante pode alterar ou
  excluir o retorno dentro das condições documentadas.
- Cargo/função de assinatura é configurado por **unidade**, não
  diretamente por usuário.
- `NivelAcesso` deve preservar público, restrito e sigiloso;
  para restrito há hipótese legal obrigatória conforme manual.
  Mudança de nível do documento pode afetar o processo.
- Tipos de processo têm níveis **permitidos** e **sugeridos**,
  restrições por órgãos/unidades e opções administrativas próprias.
  Sugestão não é autorização nem regra universal da versão SEI.

## Contratos de operação derivados

`processo.enviar`:
- Destinatários explícitos, opção `manter_aberto`, retorno programado
  opcional e confirmação de efeitos por unidade.
- Nunca presumir recebimento com o envio nem repetir em efeito incerto.

`processo.criar`:
- Tipo escolhido, assunto(s), interessado(s), especificação e
  `NivelAcesso`, validando políticas da instalação/órgão/unidade.
- Os valores permitidos não são inferidos somente da versão.

`documento.assinar`:
- Documento interno, assinante/identidade autorizada,
  cargo/função disponível na unidade e modalidade aprovada.
- A escolha visual da assinatura não prova validação criptográfica.

`documento.criar`:
- Tipo, natureza interno/externo, acesso e hipótese legal quando
  exigida. Não inventar MIME ou conteúdo de anexos ausentes.

## Testes discriminantes

1. Mesmo tipo de processo com níveis permitido e sugerido diferentes:
   sugerido não valida pedido proibido.
2. Restrito sem hipótese legal: rejeição conforme política.
3. Usuário da unidade A tenta selecionar cargo configurado apenas
   na B: não assumir disponibilidade.
4. Envio com `manter_aberto` verdadeiro/falso: comparar situação
   da unidade origem após execução.
5. Dois retornos programados no mesmo processo: não colapsar em
   `Option<Data>` global.
6. Alteração de nível de acesso do documento: verificar se afeta o
   processo, não somente o documento.

## Fontes oficiais

- [Operações básicas com processos](https://manuais.processoeletronico.gov.br/pt-br/latest/SEI/Operacoes_basicas_com_processos.html)
- [Manutenção e retorno programado](https://manuais.processoeletronico.gov.br/pt-br/latest/SEI/Operacoes_de_manutencao_e_acompanhamento_de_processos.html)
- [Operações com documentos](https://manuais.processoeletronico.gov.br/pt-br/latest/SEI/Operacoes_basicas_com_documentos.html)
- [Assinaturas das unidades](https://manuais.processoeletronico.gov.br/pt-br/latest/SEIADM/Administracao_sei/Man_Adm_SEI_04-Administracao_SEI-01_Assinatura_das_Unidades.html)
- [Tipo de processo](https://manuais.processoeletronico.gov.br/pt-br/latest/SEIADM/Administracao_sei/Man_Adm_SEI_04-Administra%C3%A7%C3%A3o_SEI-05_Tipo_de_Processo.html)

Este é um mapeamento documental, não extração dos corpos HTML das
cassettes. Antes de fixar campos Rust, comparar formulários reais,
permissões e contratos por instalação. Nenhuma alteração de runtime
é necessária para registrar estes conceitos.
